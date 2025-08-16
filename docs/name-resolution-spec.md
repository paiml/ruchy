# Ruchy Name Resolution and Scoping Rules Specification v1.0

## Abstract

This specification defines the complete name resolution and scoping semantics for Ruchy, establishing deterministic rules for identifier binding, module visibility, and Rust interoperability. The design achieves zero-ambiguity resolution with single-pass compilation where possible, falling back to fixpoint iteration only for mutually recursive definitions.

## 1. Core Resolution Model

### 1.1 Resolution Phases

Name resolution occurs in three ordered phases:

```rust
enum ResolutionPhase {
    // Phase 1: Build scope tree and collect declarations
    Collection,
    // Phase 2: Resolve imports and build import graph  
    ImportResolution,
    // Phase 3: Resolve all name references
    NameBinding,
}
```

### 1.2 Scope Hierarchy

```rust
pub enum Scope {
    // Global scope (crate root)
    Global {
        modules: HashMap<Ident, ModuleId>,
        imports: HashMap<Ident, Resolution>,
    },
    
    // Module scope (file or inline module)
    Module {
        parent: ScopeId,
        items: HashMap<Ident, ItemId>,
        private: HashSet<Ident>,
    },
    
    // Function scope
    Function {
        parent: ScopeId,
        params: Vec<(Ident, TypeId)>,
        locals: HashMap<Ident, LocalId>,
        captures: Vec<CaptureId>,
    },
    
    // Block scope (including if/match arms)
    Block {
        parent: ScopeId,
        bindings: HashMap<Ident, BindingId>,
        // Shadow parent bindings explicitly
        shadows: HashSet<Ident>,
    },
    
    // Actor message handler scope
    Handler {
        parent: ScopeId,
        message_type: TypeId,
        bindings: HashMap<Ident, BindingId>,
    }
}
```

## 2. Lexical Scoping Rules

### 2.1 Binding Introduction

Names are introduced into scope at specific syntactic positions:

```rust
// Function parameters - bind in function scope
fun process(data: Vec<i32>, threshold: i32) -> Vec<i32> {
    // 'data' and 'threshold' bound here
}

// Let bindings - bind in current block
let x = 42;           // 'x' bound from this point forward
let (a, b) = pair;    // 'a' and 'b' bound simultaneously

// Pattern matching - bind in arm scope
match value {
    Some(x) => use_x(x),  // 'x' bound only in this arm
    None => default(),
}

// For loop - bind in loop body
for item in collection {
    // 'item' bound only within loop
}
```

### 2.2 Scope Lifetime

Each scope has explicit entry and exit points:

```rust
impl ScopeRules {
    const SCOPE_ENTRY: &[SyntaxKind] = &[
        SyntaxKind::BlockExpr,
        SyntaxKind::FunctionBody,
        SyntaxKind::MatchArm,
        SyntaxKind::IfBranch,
        SyntaxKind::ForBody,
    ];
    
    const SCOPE_EXIT: &[SyntaxKind] = &[
        SyntaxKind::BlockEnd,
        SyntaxKind::Return,
        SyntaxKind::Break,
        SyntaxKind::Continue,
    ];
}
```

### 2.3 Shadowing Rules

Ruchy allows lexical shadowing with explicit rules:

```rust
// Inner scopes can shadow outer bindings
let x = 1;
{
    let x = 2;  // Shadows outer 'x'
    print(x);   // Prints 2
}
print(x);       // Prints 1

// Function parameters cannot be shadowed in same scope
fun foo(x: i32) {
    let x = 2;  // ERROR: Cannot shadow parameter 'x'
}

// Name resolver correctly binds references to shadowed variables
let data = vec![1, 2, 3];
let ref = &data;        // Binds to first 'data'
let data = vec![4, 5, 6];  // Creates new binding
// Note: Borrow checker validates lifetime safety in later phase
```

## 3. Module Resolution

### 3.1 Module Visibility Model

```rust
pub struct ModuleResolver {
    // Resolution precedence (first match wins)
    resolution_order: [ResolutionSource; 5],
}

enum ResolutionSource {
    // 1. Local scope bindings
    LocalScope,
    // 2. Module-level items  
    ModuleItems,
    // 3. Explicit imports
    Imports,
    // 4. Glob imports
    GlobImports,
    // 5. Prelude (implicit std items)
    Prelude,
}
```

### 3.2 Import Resolution Algorithm

```rust
// import tokio::spawn
// Resolves to: Resolution::External("tokio", ["spawn"])

// import ./utils/helpers
// Resolves to: Resolution::Local(PathBuf("./utils/helpers.ruchy"))

// import std::collections::{HashMap, HashSet}
// Resolves to: Resolution::Std(["collections"], ["HashMap", "HashSet"])

impl ImportResolver {
    fn resolve(&mut self, import: &Import) -> Result<Resolution> {
        match import {
            Import::Path(segments) => {
                // Check each resolution source in order
                for source in &self.resolution_order {
                    if let Some(resolution) = source.try_resolve(segments) {
                        return Ok(resolution);
                    }
                }
                Err(UnresolvedImport(segments))
            }
        }
    }
}
```

### 3.3 Import Precedence and Conflicts

```rust
// Explicit imports shadow glob imports
import std::collections::*;
import hashbrown::HashMap;  // Shadows std::collections::HashMap

// Error on conflicting explicit imports
import std::collections::HashMap;
import hashbrown::HashMap;  // ERROR: Conflicting import

// Module-local items shadow imports
import external::process;
fun process() {}  // Shadows the import within this module
```

## 4. Method Resolution Order

### 4.1 Resolution Priority

Method resolution follows strict precedence:

```rust
enum MethodResolution {
    // 1. Inherent methods (defined directly on type)
    Inherent,
    // 2. Trait methods in scope (explicit import)
    TraitExplicit,
    // 3. Trait methods via glob import
    TraitGlob,
    // 4. Extension methods (Ruchy-specific)
    Extension,
}

// Extension methods enable retrofitting types with new behavior
// without modifying original definition or creating wrapper types
```

### 4.1.1 Extension Method Scoping

```rust
// Definition: extensions/vec_ext.ruchy
extension on Vec<T> {
    fun sum(self) -> T where T: Add {
        self.fold(T::zero(), |a, b| a + b)
    }
}

// Usage: main.ruchy
import extensions::vec_ext;  // Brings extension into scope

fun main() {
    let nums = vec![1, 2, 3];
    let total = nums.sum();  // Resolves to extension method
}
```

Extension resolution rules:
- Extensions must be explicitly imported or defined in current module
- Extensions cannot override inherent methods
- Multiple extensions defining same method = error at import

impl MethodResolver {
    fn resolve_method(&self, receiver: &Type, name: &Ident) -> Resolution {
        // Try inherent methods first
        if let Some(method) = self.inherent_methods(receiver, name) {
            return Resolution::Inherent(method);
        }
        
        // Check traits in scope
        for trait_id in self.traits_in_scope() {
            if let Some(method) = self.trait_method(trait_id, receiver, name) {
                return Resolution::Trait(trait_id, method);
            }
        }
        
        Resolution::NotFound
    }
}
```

### 4.2 Pipeline Resolution

Pipeline operators transform into nested function calls with unambiguous resolution:

```rust
// data |> process |> filter(threshold) |> collect
// Parses as: data |> process |> (filter(threshold)) |> collect
// Resolves to: collect(filter(threshold)(process(data)))

impl PipelineResolver {
    fn resolve_pipeline(&self, expr: &Pipeline) -> Result<Expr> {
        let mut current = expr.initial;
        
        for stage in &expr.stages {
            // Each stage is a complete expression
            current = match self.resolve_expr(stage)? {
                // Function value - apply to current
                Expr::Function(func) => {
                    Expr::Call(func, vec![current])
                },
                // Partially applied function - apply to current
                Expr::PartialApp(func, args) => {
                    let mut full_args = vec![current];
                    full_args.extend(args);
                    Expr::Call(func, full_args)
                },
                // Method reference - call on current
                Expr::MethodRef(method) => {
                    Expr::MethodCall(current, method, vec![])
                },
                _ => return Err(InvalidPipelineStage),
            }
        }
        Ok(current)
    }
}
```

## 5. Actor Message Handler Resolution

### 5.1 Message Pattern Scoping

Actor message handlers create implicit scopes:

```rust
actor Counter {
    var count = 0  // Actor-level state
    
    receive {
        // Each pattern creates new scope
        Increment(n) => {
            // 'n' bound only here
            count += n
        }
        
        GetCount(reply) => {
            // 'reply' bound only here
            reply ! count
        }
    }
}
```

### 5.2 Actor State Visibility

```rust
impl ActorScope {
    fn resolve(&self, name: &Ident) -> Resolution {
        // Priority order:
        // 1. Message pattern bindings
        if let Some(binding) = self.pattern_bindings.get(name) {
            return Resolution::PatternBinding(binding);
        }
        
        // 2. Actor state (self.field)
        if let Some(field) = self.actor_fields.get(name) {
            return Resolution::ActorField(field);
        }
        
        // 3. Parent module scope
        self.parent_scope.resolve(name)
    }
}
```

## 6. Rust Interop Resolution

### 6.1 Trait Method Disambiguation

When Rust traits are imported, explicit disambiguation may be required:

```rust
// Both traits define 'process' method
import rust_crate::{TraitA, TraitB};

let result = value.process();  // ERROR: Ambiguous

// Explicit disambiguation
let result = TraitA::process(&value);  // OK
let result = value.process::<TraitA>();  // OK (UFCS)
```

### 6.2 Visibility Mapping

```rust
pub struct VisibilityRules {
    // Ruchy visibility -> Rust visibility
    mapping: VisibilityMapping,
}

enum VisibilityMapping {
    // pub in Ruchy -> pub in Rust
    Public => "pub",
    // No modifier -> pub(crate)
    ModuleLocal => "pub(crate)",  
    // priv -> private (no modifier)
    Private => "",
}
```

## 7. Hygiene Rules

### 7.1 Macro Hygiene (Future)

Establish hygiene rules for future macro system:

```rust
pub struct HygieneContext {
    // Each macro expansion gets unique SyntaxContext
    syntax_contexts: Vec<SyntaxContext>,
    // Mark prevents capture
    marks: HashMap<Ident, Mark>,
}

// Hygienic binding - cannot capture user names
macro_rules! safe_let {
    ($e:expr) => {
        let __temp = $e;  // __temp cannot collide
    }
}
```

### 7.2 Compiler-Generated Names

```rust
impl NameGenerator {
    // Compiler-generated names use reserved prefix
    const RESERVED_PREFIX: &str = "__ruchy_";
    
    fn generate_unique(&mut self, hint: &str) -> Ident {
        let name = format!("{}{}{}", 
            Self::RESERVED_PREFIX, 
            hint, 
            self.counter);
        self.counter += 1;
        Ident::new(name)
    }
}
```

## 8. Error Reporting

### 8.1 Resolution Errors

Each resolution failure produces structured diagnostics:

```rust
pub enum ResolutionError {
    UnresolvedName {
        name: Ident,
        scope: ScopeId,
        suggestions: Vec<Ident>,  // Did you mean?
    },
    
    AmbiguousName {
        name: Ident,
        candidates: Vec<Resolution>,
        note: String,  // How to disambiguate
    },
    
    PrivateInPublic {
        private_item: ItemId,
        public_context: ItemId,
    },
    
    CyclicImport {
        cycle: Vec<ModulePath>,
    },
}
```

### 8.2 Error Recovery

Parser continues resolution after errors:

```rust
impl ErrorRecovery {
    fn recover_from_unresolved(&mut self, name: &Ident) -> Resolution {
        // Insert synthetic binding to continue
        self.synthetic_bindings.insert(name.clone());
        Resolution::Synthetic(self.generate_synthetic_type())
    }
}
```

## 9. Performance Characteristics

### 9.1 Complexity Guarantees

- Name lookup: O(1) average via HashMap
- Import resolution: O(n) where n = import depth  
- Method resolution: O(t) where t = traits in scope
- Full crate resolution: O(n log n) where n = total names

### 9.2 Incremental Resolution

```rust
pub struct IncrementalResolver {
    // Only re-resolve changed modules
    dirty: HashSet<ModuleId>,
    // Cache unchanged resolutions
    resolution_cache: HashMap<(Ident, ScopeId), Resolution>,
}
```

## 10. Validation Rules

### 10.1 Invariants

The resolver maintains these invariants:

1. Every name reference resolves to exactly one binding
2. No cycles in import graph
3. Private items not visible outside module
4. **Phase Separation**: Name resolution produces binding information; borrow checking validates lifetime safety
5. Method resolution is deterministic

### 10.2 Static Checks

```rust
impl Validator {
    fn validate(&self, resolution_map: &ResolutionMap) -> Result<()> {
        self.check_no_unresolved_names()?;
        self.check_no_import_cycles()?;
        self.check_visibility_consistency()?;
        self.check_orphan_rules()?;
        // Note: Lifetime safety validated in borrow checking phase
        Ok(())
    }
}
```

## Appendix: Resolution Examples

### Complex Resolution Chain

```rust
// libs/data.ruchy
pub fun process(x: i32) -> i32 { x * 2 }

// main.ruchy
import libs::data::process as data_process

fun process(x: i32) -> i32 { x + 1 }

fun main() {
    let x = 10;
    
    {
        let process = |x| x * 3;  // Shadows function
        let a = process(x);       // Calls closure: 30
    }
    
    let b = process(x);          // Calls function: 11
    let c = data_process(x);     // Calls import: 20
}
```

This example demonstrates:
1. Import aliasing
2. Function-level name binding
3. Lexical shadowing with closure
4. Resolution to correct binding at each callsite