# Ruchy Edge Cases and Implementation Specification

## Executive Summary

This document codifies the mechanical implementation details for Ruchy's compiler, addressing critical edge cases in module resolution, ownership transfer, effect polymorphism, and trait resolution. Each section provides deterministic rules and concrete data structures required for prototype implementation.

## 1. Module Resolution: Canonical Mapping

### 1.1 Core Data Structures

```rust
pub struct ModuleResolver {
    // Source of truth for module mappings
    module_cache: ModuleCache,
    // Dependency graph for incremental compilation
    dep_graph: PetGraph<ModuleId, DependencyEdge>,
    // Maps import paths to generated Rust paths
    path_mappings: HashMap<ImportPath, RustPath>,
}

pub struct ModuleCache {
    // External crates from Cargo.toml
    cargo_crates: HashMap<String, CrateMetadata>,
    // Local .ruchy files with their AST and symbols
    local_modules: HashMap<PathBuf, ParsedModule>,
    // REPL-defined modules (session-local)
    repl_modules: HashMap<String, CompiledModule>,
}

pub struct CrateMetadata {
    version: semver::Version,
    features: Vec<String>,
    exported_items: HashSet<String>,
    trait_impls: Vec<TraitImpl>,
}
```

### 1.2 Resolution Algorithm

```rust
impl ModuleResolver {
    fn resolve_import(&mut self, import: &Import) -> Result<Resolution> {
        match import {
            // Direct crate reference: `import tokio::spawn`
            Import::External(crate_name, path) => {
                if let Some(metadata) = self.cargo_crates.get(crate_name) {
                    Ok(Resolution::External {
                        rust_use: format!("use {}::{};", crate_name, path),
                        requires_trait: self.check_trait_requirement(metadata, path),
                    })
                } else {
                    Err(Error::UnknownCrate(crate_name.clone()))
                }
            },
            
            // Relative import: `import ./analytics`
            Import::Relative(path) => {
                let canonical = self.canonicalize_path(path)?;
                let module_name = self.generate_module_name(&canonical);
                
                Ok(Resolution::Local {
                    rust_mod: format!("mod {};", module_name),
                    rust_use: format!("use {}::*;", module_name),
                    source_path: canonical,
                })
            },
            
            // Standard library: `import std::fs`
            Import::Std(path) => {
                Ok(Resolution::Std {
                    rust_use: format!("use std::{};", path),
                })
            },
        }
    }
}
```

### 1.3 Edge Cases

**Circular Dependencies:**
```rust
// a.ruchy imports b.ruchy imports a.ruchy
// Detection via topological sort in dep_graph
if self.dep_graph.is_cyclic() {
    return Err(Error::CircularDependency(cycle_path));
}
```

**Diamond Dependencies:**
```rust
// a imports b,c; both b,c import d
// Resolution: d compiled once, shared via module_cache
let shared_module = self.module_cache.get_or_compile(d_path)?;
```

**Version Conflicts:**
```rust
// Two dependencies require different versions of same crate
// Resolution: Follow Cargo's semver compatibility rules
// Major version differences = error
// Minor/patch differences = use newer version
```

## 2. Effect System: Polymorphic Composition

### 2.1 Effect Algebra

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effects(u16);

impl Effects {
    const PURE: u16    = 0x0000;
    const ASYNC: u16   = 0x0001;
    const IO: u16      = 0x0002;
    const UNSAFE: u16  = 0x0004;
    const ALLOC: u16   = 0x0008;
    const PANIC: u16   = 0x0010;
    const DIVERGE: u16 = 0x0020;
    const MCP: u16     = 0x0040;
    
    // Lattice operations
    pub fn join(self, other: Effects) -> Effects {
        Effects(self.0 | other.0)
    }
    
    pub fn meet(self, other: Effects) -> Effects {
        Effects(self.0 & other.0)
    }
    
    // Subsumption check: can `self` be used where `required` is expected?
    pub fn subsumes(self, required: Effects) -> bool {
        (self.0 & required.0) == required.0
    }
}
```

### 2.2 Effect Inference

```rust
pub struct EffectInference {
    // Maps expressions to their inferred effects
    effect_map: HashMap<ExprId, Effects>,
    // Effect variables for polymorphic functions
    effect_vars: HashMap<EffectVar, Effects>,
}

impl EffectInference {
    fn infer_expr(&mut self, expr: &Expr) -> Effects {
        match expr {
            Expr::Call(func, args) => {
                let func_effects = self.lookup_function_effects(func);
                let arg_effects = args.iter()
                    .map(|arg| self.infer_expr(arg))
                    .fold(Effects::PURE, Effects::join);
                func_effects.join(arg_effects)
            },
            
            Expr::Await(inner) => {
                Effects::ASYNC.join(self.infer_expr(inner))
            },
            
            Expr::UnsafeBlock(block) => {
                Effects::UNSAFE.join(self.infer_block(block))
            },
            
            Expr::FileRead(_) => Effects::IO,
            
            Expr::Panic(_) => Effects::PANIC.join(Effects::DIVERGE),
            
            _ => Effects::PURE,
        }
    }
}
```

### 2.3 Effect Polymorphism

```rust
// Higher-order function with polymorphic effects
// map :: (a -[e]-> b) -> [a] -[e]-> [b]
pub struct PolymorphicEffect {
    var: EffectVar,
    constraints: Vec<EffectConstraint>,
}

// Example: map inherits effects from its function argument
fn type_map_function() -> FunctionType {
    let effect_var = EffectVar::fresh();
    FunctionType {
        params: vec![
            Type::Function(Box::new(Type::Var(0)), 
                          Box::new(Type::Var(1)), 
                          effect_var),
            Type::List(Box::new(Type::Var(0))),
        ],
        returns: Type::List(Box::new(Type::Var(1))),
        effects: PolymorphicEffect { var: effect_var, constraints: vec![] },
    }
}
```

## 3. Ownership Transfer: Deterministic Rules

### 3.1 Ownership Analysis

```rust
pub struct OwnershipAnalyzer {
    // Lifetime ranges for each binding
    lifetimes: HashMap<BindingId, LifetimeRange>,
    // Usage patterns for optimization
    usage_patterns: HashMap<BindingId, UsagePattern>,
    // Escape analysis results
    escape_info: HashMap<BindingId, EscapeInfo>,
}

#[derive(Debug)]
pub enum UsagePattern {
    SingleUse,           // Used exactly once
    SharedImmutable,     // Multiple reads, no writes
    Mutated,            // At least one mutable use
    Escaped,            // Captured by closure or returned
}

pub struct LifetimeRange {
    definition: Location,
    last_use: Location,
    uses: Vec<(Location, UseKind)>,
}
```

### 3.2 Transfer Rules

```rust
impl OwnershipAnalyzer {
    fn analyze_transfer(&mut self, expr: &Assignment) -> TransferStrategy {
        let source_id = self.resolve_binding(&expr.source);
        let pattern = &self.usage_patterns[&source_id];
        
        match pattern {
            UsagePattern::SingleUse => {
                // Move: this is the only use
                TransferStrategy::Move
            },
            
            UsagePattern::SharedImmutable if !self.crosses_await_point(source_id) => {
                // Borrow: safe to share reference
                TransferStrategy::Borrow
            },
            
            UsagePattern::SharedImmutable => {
                // Clone: needed across await boundary
                TransferStrategy::Clone
            },
            
            UsagePattern::Mutated => {
                // Clone: preserve independence
                TransferStrategy::Clone
            },
            
            UsagePattern::Escaped => {
                // Rc/Arc: shared ownership required
                if self.is_thread_boundary(source_id) {
                    TransferStrategy::Arc
                } else {
                    TransferStrategy::Rc
                }
            },
        }
    }
}
```

### 3.3 REPL vs Compiled Semantics

```rust
pub enum ExecutionContext {
    REPL {
        // Everything Rc'd for persistence
        default_strategy: TransferStrategy::Rc,
    },
    Script {
        // Optimize for single-pass execution
        default_strategy: TransferStrategy::Move,
    },
    Compiled {
        // Full optimization enabled
        default_strategy: TransferStrategy::Inferred,
    },
}
```

## 4. Trait Resolution: Method Dispatch

### 4.1 Trait Registry

```rust
pub struct TraitRegistry {
    // Method name -> Traits that define it
    method_index: HashMap<String, Vec<TraitDef>>,
    // Type -> Implemented traits
    impl_index: HashMap<TypeId, HashSet<TraitId>>,
    // Orphan rule checker
    orphan_checker: OrphanChecker,
}

pub struct TraitDef {
    path: String,           // e.g., "std::iter::Iterator"
    methods: Vec<MethodSig>,
    associated_types: Vec<String>,
    supertraits: Vec<TraitId>,
}
```

### 4.2 Resolution Algorithm

```rust
impl TraitRegistry {
    fn resolve_method_call(&self, 
                          receiver_type: &Type, 
                          method: &str,
                          args: &[Type]) -> Result<MethodResolution> {
        // Step 1: Find candidate traits
        let candidates = self.method_index.get(method)
            .ok_or(Error::UnknownMethod(method.to_string()))?;
        
        // Step 2: Filter by receiver type
        let applicable: Vec<_> = candidates.iter()
            .filter(|trait_def| {
                self.type_implements_trait(receiver_type, trait_def.id)
            })
            .collect();
        
        // Step 3: Resolve ambiguity
        match applicable.len() {
            0 => Err(Error::NoImplementation),
            1 => Ok(MethodResolution {
                trait_path: applicable[0].path.clone(),
                needs_deref: self.compute_deref_chain(receiver_type, applicable[0]),
            }),
            _ => {
                // Ambiguous: use argument types to disambiguate
                self.disambiguate_by_args(&applicable, args)
            }
        }
    }
}
```

### 4.3 Auto-Import Generation

```rust
pub struct ImportInjector {
    used_traits: HashSet<String>,
    prelude_traits: HashSet<String>,
}

impl ImportInjector {
    fn generate_imports(&self) -> Vec<String> {
        self.used_traits
            .difference(&self.prelude_traits)
            .map(|trait_path| format!("use {};", trait_path))
            .collect()
    }
}
```

## 5. Error Propagation Model

### 5.1 Result Type Inference

```rust
pub struct ErrorInference {
    // Functions using ? operator
    fallible_functions: HashSet<FunctionId>,
    // Unified error types per function
    error_types: HashMap<FunctionId, ErrorType>,
}

impl ErrorInference {
    fn infer_return_type(&mut self, func: &Function) -> Type {
        if self.uses_try_operator(func) {
            let error_type = self.unify_error_types(func);
            Type::Result(Box::new(func.declared_return), error_type)
        } else {
            func.declared_return.clone()
        }
    }
    
    fn unify_error_types(&mut self, func: &Function) -> ErrorType {
        let error_sources = self.collect_error_sources(func);
        
        match error_sources.len() {
            0 => unreachable!(),
            1 => error_sources[0].clone(),
            _ => {
                // Multiple error types: find common trait
                if let Some(common) = self.find_common_error_trait(&error_sources) {
                    ErrorType::Trait(common)
                } else {
                    // Fallback: trait object
                    ErrorType::Dynamic("Box<dyn std::error::Error>".into())
                }
            }
        }
    }
}
```

### 5.2 Panic Boundaries

```rust
pub enum ErrorStrategy {
    Panic,      // Unrecoverable: index out of bounds
    Result,     // Recoverable: file not found
    Option,     // Absence: map lookup
}

impl ErrorStrategy {
    fn from_context(expr: &Expr) -> Self {
        match expr {
            Expr::Index(_, _) => ErrorStrategy::Panic,
            Expr::FileOp(_) => ErrorStrategy::Result,
            Expr::MapGet(_, _) => ErrorStrategy::Option,
            _ => ErrorStrategy::Result,
        }
    }
}
```

## 6. Incremental Compilation Cache

### 6.1 Cache Architecture

```rust
pub struct CompilationCache {
    // AST -> TypedAST cache
    typed_cache: DashMap<AstHash, Arc<TypedAst>>,
    // TypedAST -> Rust source cache
    rust_cache: DashMap<TypedAstHash, Arc<String>>,
    // Compiled artifacts (.so/.dll)
    binary_cache: DashMap<RustHash, Arc<Library>>,
    // Dependency tracking
    dep_tracker: DependencyTracker,
}

pub struct DependencyTracker {
    // Which items depend on which
    forward_deps: HashMap<ItemId, HashSet<ItemId>>,
    // Reverse dependency for invalidation
    reverse_deps: HashMap<ItemId, HashSet<ItemId>>,
    // Fingerprints for change detection
    fingerprints: HashMap<ItemId, u64>,
}
```

### 6.2 Invalidation Strategy

```rust
impl CompilationCache {
    fn invalidate(&mut self, changed: ItemId) {
        let mut invalidation_queue = VecDeque::new();
        invalidation_queue.push_back(changed);
        
        while let Some(item) = invalidation_queue.pop_front() {
            // Remove from caches
            self.typed_cache.remove(&item);
            self.rust_cache.remove(&item);
            self.binary_cache.remove(&item);
            
            // Queue dependents
            if let Some(deps) = self.dep_tracker.reverse_deps.get(&item) {
                invalidation_queue.extend(deps.iter().cloned());
            }
        }
    }
    
    fn invalidate_repl_definition(&mut self, name: &str) {
        // REPL definitions form a virtual module with dynamic dependencies
        let repl_module = ItemId::ReplModule(name.to_string());
        
        // Track REPL inter-dependencies via name resolution
        let affected = self.dep_tracker
            .find_repl_references(name)
            .into_iter()
            .map(|def| ItemId::ReplModule(def));
        
        // Cascade invalidation through REPL dependency graph
        for item in affected {
            self.invalidate(item);
        }
        
        // Rebuild affected REPL definitions lazily on next access
        self.repl_rebuild_queue.insert(name.to_string());
    }
}
```

## 7. Runtime Bridge

### 7.1 FFI Marshalling

```rust
pub struct RuntimeBridge {
    // Loaded Rust dynamic libraries
    libraries: HashMap<ModuleId, Library>,
    // Symbol resolution cache
    symbols: HashMap<String, Symbol>,
    // Type marshalling strategies
    marshallers: HashMap<TypeId, Marshaller>,
}

pub enum Marshaller {
    Direct,                    // POD types
    Clone,                     // String, Vec
    Serialize(SerdeFormat),   // Complex structures
    Opaque(OpaqueHandle),     // Rust-only types
}

impl RuntimeBridge {
    fn call_rust_function(&mut self, 
                         name: &str, 
                         args: Vec<Value>) -> Result<Value> {
        let symbol = self.symbols.get(name)
            .ok_or(Error::UnresolvedSymbol)?;
        
        let marshalled_args = args.into_iter()
            .map(|arg| self.marshal_to_rust(arg))
            .collect::<Result<Vec<_>>>()?;
        
        let result = unsafe {
            symbol.call(marshalled_args)
        };
        
        self.marshal_from_rust(result)
    }
}
```

## 8. Build Pipeline

### 8.1 Cargo.toml Generation

```rust
pub struct CargoGenerator {
    base_deps: BTreeMap<String, Dependency>,
    feature_flags: HashMap<String, Vec<String>>,
}

impl CargoGenerator {
    fn generate_cargo_toml(&self, imports: &[Import]) -> String {
        let mut deps = self.base_deps.clone();
        
        // Add dependencies from imports
        for import in imports {
            if let Import::External(crate_name, _) = import {
                deps.entry(crate_name.clone())
                    .or_insert(Dependency::latest());
            }
        }
        
        toml::to_string(&CargoManifest {
            package: PackageInfo {
                name: format!("ruchy_generated_{}", self.hash()),
                version: "0.1.0",
                edition: "2021",
            },
            dependencies: deps,
            features: self.feature_flags.clone(),
        }).unwrap()
    }
}
```

### 8.2 Crate Metadata Extraction

```rust
pub struct MetadataExtractor {
    rustdoc_cache: PathBuf,
    metadata_index: HashMap<String, CrateMetadata>,
}

impl MetadataExtractor {
    fn extract_crate_metadata(&mut self, crate_name: &str) -> Result<CrateMetadata> {
        // Strategy 1: Use cargo-metadata for dependency resolution
        let cargo_meta = MetadataCommand::new()
            .exec()
            .map_err(|e| Error::CargoMetadata(e))?;
        
        // Strategy 2: Generate rustdoc JSON for trait/type information
        let rustdoc_output = Command::new("cargo")
            .args(&["rustdoc", "--", "-Z", "unstable-options", 
                   "--output-format", "json"])
            .output()?;
        
        // Parse rustdoc JSON for exported items and trait impls
        let doc_data: RustdocJson = serde_json::from_slice(&rustdoc_output.stdout)?;
        
        Ok(CrateMetadata {
            version: cargo_meta.packages
                .iter()
                .find(|p| p.name == crate_name)
                .map(|p| p.version.clone())
                .unwrap_or_default(),
            exported_items: self.extract_exports(&doc_data),
            trait_impls: self.extract_impls(&doc_data),
            features: cargo_meta.resolve
                .and_then(|r| r.features.get(crate_name).cloned())
                .unwrap_or_default(),
        })
    }
    
    // Run after cargo fetch, before first compilation
    fn index_all_dependencies(&mut self) -> Result<()> {
        let manifest = cargo_metadata::MetadataCommand::new().exec()?;
        
        for package in manifest.packages {
            if package.source.is_some() { // External dependency
                let metadata = self.extract_crate_metadata(&package.name)?;
                self.metadata_index.insert(package.name, metadata);
            }
        }
        Ok(())
    }
}
```

## 9. Macro Hygiene

### 9.1 Macro Boundary

```rust
pub struct MacroHandler {
    // Known Rust macros available to Ruchy
    available_macros: HashSet<String>,
    // Hygiene context for generated code
    hygiene_ctx: HygieneContext,
}

impl MacroHandler {
    fn handle_macro_call(&self, macro_call: &MacroCall) -> Result<TokenStream> {
        // Verify macro is available
        if !self.available_macros.contains(&macro_call.name) {
            return Err(Error::UnknownMacro(macro_call.name.clone()));
        }
        
        // Convert Ruchy args to Rust TokenStream
        let tokens = self.args_to_tokens(&macro_call.args)?;
        
        // Preserve hygiene markers
        self.hygiene_ctx.mark(tokens)
    }
}

// Prelude macros available by default
const PRELUDE_MACROS: &[&str] = &[
    "println!", "format!", "vec!", "assert!", 
    "debug_assert!", "todo!", "unimplemented!",
];
```

## 10. Diagnostic Philosophy

### 10.1 Error Message Design

Every compiler error maps to a user-facing diagnostic following mechanical transparency principles:

```rust
pub struct Diagnostic {
    // What rule was violated
    rule: CompilerRule,
    // Why it matters
    rationale: &'static str,
    // How to fix it
    suggestions: Vec<FixSuggestion>,
    // Show the transformation that failed
    transformation: Option<TransformationTrace>,
}

impl Diagnostic {
    fn render(&self) -> String {
        format!(
            "Error: {}\n\
             Rule: {}\n\
             Why: {}\n\
             Fix: {}\n\
             Transformation: {}",
            self.message(),
            self.rule.explain(),
            self.rationale,
            self.suggestions.join(" or "),
            self.transformation.map(|t| t.show()).unwrap_or_default()
        )
    }
}
```

### 10.2 Examples

**Module Resolution Error:**
```
Error: Cannot resolve module 'analytics'
Rule: Modules must exist as .ruchy files or Cargo dependencies
Why: No file './analytics.ruchy' and 'analytics' not in Cargo.toml
Fix: Create './analytics.ruchy' or run 'ruchy add analytics'
Transformation: import analytics → [module resolution failed]
```

**Effect Violation:**
```
Error: Pure function cannot call async function
Rule: Effects must be declared or inferred
Why: 'calculate' is pure but calls 'fetch_data' which is async
Fix: Mark 'calculate' as async or use 'fetch_data_sync'
Transformation: Effects(PURE) ⊈ Effects(ASYNC)
```

**Ownership Conflict:**
```
Error: Value used after move
Rule: Non-Copy values have single ownership
Why: 'data' moved to 'process' but used again on line 15
Fix: Clone before move: 'process(data.clone())' or borrow: 'process(&data)'
Transformation: Move(data) → InvalidateBinding(data) → UseAfterMove
```

## Conclusion

This specification provides the mechanical foundation for Ruchy's implementation. Each subsystem has deterministic rules, concrete data structures, and clear boundaries. The design eliminates ambiguity while preserving mechanical transparency: every transformation from Ruchy to Rust is observable, predictable, and optimizable.

The path forward:
1. Implement parser with these AST nodes
2. Build type inference with effect tracking
3. Create ownership analyzer
4. Generate Rust with trait resolution
5. Link via runtime bridge

Total estimated complexity: ~15K LOC for MVP, achieving Python ergonomics with zero-cost Rust performance.