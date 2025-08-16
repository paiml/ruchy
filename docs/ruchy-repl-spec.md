# Ruchy REPL Full Specification v2.0

## 1. Overview

The Ruchy REPL provides an interactive programming environment that bridges scripting ergonomics with systems performance. It operates as a hybrid interpreter-compiler, maintaining sub-15ms response times for interactive work while transparently compiling hot paths to native code.

## 2. Architecture

### 2.1 Core Components

```rust
pub struct ReplCore {
    frontend: FrontendManager,
    session: SessionState,
    executor: ExecutionEngine,
    compiler: CompilationCache,
    introspector: IntrospectionService,
    runtime: TokioRuntime,
    security: SecurityContext,  // NEW: Sandbox management
}
```

### 2.2 Execution Pipeline

```
Input → Lex → Parse → Type Check → Execute → Display
                ↓         ↓           ↓
            [Cache]   [Inference]  [JIT/Interp]
                          ↓
                   [Dep Graph Update]
```

## 3. Frontend Specification

### 3.1 Mode System

The REPL operates in six distinct modes, each with dedicated parsing and execution semantics:

```rust
enum ReplMode {
    Ruchy,      // Default: code execution
    Cargo,      // ]: package management  
    Shell,      // ;: system commands
    Help,       // ?: documentation
    Mcp,        // @: LLM tool interaction (async, non-blocking)
    Search,     // /: history search
}
```

Mode transitions occur via prefix characters at line start:

```
ruchy> let x = 42
ruchy> ] add tokio
cargo> ; ls -la
shell> ? Vec::push
help> @ generate tests for last function
mcp> [streaming...] Here's a comprehensive test suite...
ruchy> / previous function
search>
```

#### 3.1.1 MCP Mode Operation

MCP mode operates asynchronously to prevent blocking:

```rust
impl McpMode {
    pub async fn handle_request(&mut self, prompt: &str) {
        let request_id = self.submit_request(prompt).await;
        
        // Return control immediately
        self.frontend.show_prompt();
        
        // Stream responses as they arrive
        tokio::spawn(async move {
            while let Some(chunk) = self.stream_response(request_id).await {
                println!("[mcp:{}] {}", request_id, chunk);
            }
        });
    }
}
```

### 3.2 Input Handling

```rust
impl InputProcessor {
    fn process(&mut self, line: &str) -> Command {
        match line.chars().next() {
            Some(']') => self.enter_mode(Mode::Cargo, &line[1..]),
            Some(';') => self.execute_shell(&line[1..]),
            Some('?') => self.show_help(&line[1..]),
            Some('@') => self.mcp_request(&line[1..]),
            Some('/') => self.search_history(&line[1..]),
            _ => self.execute_ruchy(line),
        }
    }
}
```

### 3.3 Line Editing

Built on `rustyline` with custom completions:

- Tab completion for identifiers, paths, and methods
- Bracket matching with highlighting
- Multi-line input with proper indentation
- Vi/Emacs key bindings

## 4. Session State Management

### 4.1 Environment Model

```rust
pub struct SessionState {
    bindings: HashMap<Ident, Binding>,
    types: TypeEnvironment,
    results: ResultHistory,
    metadata: SessionMetadata,
}

pub struct Binding {
    value: Value,
    ty: Type,
    mutable: bool,
    generation: u64,  // For redefinition tracking
}

pub struct TypeEnvironment {
    types: HashMap<TypeId, TypeDef>,
    type_generation: u64,  // NEW: Type versioning
}
```

### 4.2 Result History

Results are automatically bound to underscore variables:

```rust
impl ResultHistory {
    pub fn push(&mut self, value: Value) -> Ident {
        self.current = value.clone();
        let index = self.history.len();
        self.history.push(value);
        
        // Bind to _ and _N
        self.bind_underscore(index)
    }
}
```

Usage:
```
ruchy> 2 + 2
4
ruchy> _ * 10
40
ruchy> _1 + _2
44
```

### 4.3 Persistence

History persists to SQLite with full-text search:

```sql
CREATE TABLE repl_history (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    command TEXT NOT NULL,
    result TEXT,
    mode TEXT NOT NULL,
    project_id TEXT,
    success BOOLEAN,
    trust_level INTEGER DEFAULT 0  -- NEW: Security tracking
);

CREATE VIRTUAL TABLE history_fts USING fts5(
    command, result, content=repl_history
);
```

## 5. Execution Engine

### 5.1 Hybrid Execution Model

```rust
pub enum ExecutionStrategy {
    Interpret,      // Cold code: tree-walk
    JitCompile,     // Hot code: Cranelift
    AotCached,      // Cached: pre-compiled
}

impl ExecutionEngine {
    pub fn execute(&mut self, ast: &Ast) -> Result<Value> {
        let strategy = self.select_strategy(ast);
        
        match strategy {
            Interpret => self.interpret(ast),
            JitCompile => self.jit_execute(ast),
            AotCached => self.load_cached(ast),
        }
    }
    
    fn select_strategy(&self, ast: &Ast) -> ExecutionStrategy {
        if let Some(cached) = self.cache.get(ast) {
            return AotCached;
        }
        
        if self.profile.is_hot(ast) {
            return JitCompile;
        }
        
        Interpret
    }
}
```

### 5.2 JIT Compilation Thresholds

```rust
pub struct JitProfile {
    execution_counts: HashMap<AstId, u32>,
    execution_times: HashMap<AstId, Duration>,
    
    const HOT_COUNT: u32 = 100;
    const HOT_TIME_MS: u64 = 50;
}
```

### 5.3 Async Integration

Top-level await is automatic:

```rust
impl Executor {
    pub fn eval_top_level(&mut self, expr: Expr) -> Value {
        match expr.ty() {
            Type::Future(inner) => {
                self.runtime.block_on(async {
                    self.eval_async(expr).await
                })
            }
            _ => self.eval_sync(expr),
        }
    }
}
```

## 6. Compilation & Caching

### 6.1 Incremental Compilation

```rust
pub struct CompilationCache {
    mem_cache: HashMap<AstHash, CompiledArtifact>,
    disk_cache: DiskCache,
    dep_graph: DependencyGraph,
}

impl CompilationCache {
    pub fn compile_incremental(&mut self, ast: &Ast) -> CompiledArtifact {
        let affected = self.dep_graph.invalidated_by(ast);
        
        for node in affected {
            self.mem_cache.remove(&node);
        }
        
        self.compile_minimal(ast)
    }
}
```

#### 6.1.1 Dependency Graph Construction

The dependency graph tracks three types of dependencies:

```rust
enum Dependency {
    NameRef(Ident),      // Variable/function reference
    TypeRef(TypeId),     // Type reference
    TraitImpl(TraitId),  // Trait implementation
}

impl DependencyGraph {
    pub fn build(&mut self, ast: &Ast) {
        // Walk AST collecting all identifier references
        ast.visit_identifiers(|id| {
            self.add_edge(ast.id(), id);
        });
        
        // Track type dependencies
        ast.visit_type_refs(|ty_id| {
            self.add_type_edge(ast.id(), ty_id);
        });
    }
}
```

### 6.2 Cache Structure

```
~/.ruchy/cache/
├── modules/
│   ├── 7a8f9b2c.so    # Compiled modules
│   └── metadata.db     # Module metadata
├── functions/
│   ├── abc123de.mir    # MIR cache
│   └── abc123de.o      # Object files
└── session/
    └── current.db      # Live session state
```

### 6.3 Zero-Copy FFI

```rust
impl ModuleLoader {
    pub fn load_crate(&mut self, name: &str) -> Result<Module> {
        // Security check
        self.security.verify_crate_allowed(name)?;
        
        // Check if already compiled
        if let Some(dylib) = self.find_cached(name) {
            return self.load_dylib(dylib);
        }
        
        // Build as dynamic library
        let dylib = self.build_crate_dylib(name)?;
        
        // Load symbols without copying
        unsafe {
            let lib = libloading::Library::new(dylib)?;
            self.extract_symbols(lib)
        }
    }
}
```

## 7. Introspection System

### 7.1 Query Operators

```rust
enum IntrospectionOp {
    Type,       // :t expr
    Doc,        // expr?
    Source,     // expr??
    Assembly,   // expr???
    Traits,     // :i Type
}
```

### 7.2 Implementation

```rust
impl IntrospectionService {
    pub fn introspect(&self, op: IntrospectionOp, target: &str) -> String {
        match op {
            Type => self.show_type(target),
            Doc => self.show_docs(target),
            Source => self.show_source(target),
            Assembly => self.show_mir(target),
            Traits => self.show_impls(target),
        }
    }
    
    fn show_mir(&self, target: &str) -> String {
        let artifact = self.cache.get_artifact(target)?;
        artifact.mir_repr()
    }
}
```

## 8. Display System

### 8.1 Smart Rendering

```rust
trait ReplDisplay {
    fn repl_format(&self, ctx: &DisplayContext) -> DisplayOutput;
}

impl ReplDisplay for DataFrame {
    fn repl_format(&self, ctx: &DisplayContext) -> DisplayOutput {
        if self.rows() > ctx.terminal_height {
            self.paginated_display()
        } else {
            self.tabular_display()
        }
    }
}
```

### 8.2 Output Suppression

Semicolons suppress output; loops never auto-print:

```rust
impl OutputController {
    fn should_display(&self, stmt: &Statement) -> bool {
        !stmt.ends_with_semicolon() 
        && !stmt.is_in_loop()
        && !stmt.is_assignment()
    }
}
```

## 9. Redefinition Semantics

### 9.1 Variable Redefinition

```rust
impl Redefinition {
    pub fn redefine(&mut self, name: Ident, value: Value) {
        let gen = self.next_generation();
        
        // New code sees new definition
        self.current.insert(name, (value, gen));
        
        // Old compiled code retains old binding
        // via generation tracking in closures
    }
}
```

### 9.2 Type Redefinition

Type redefinition requires cache invalidation:

```rust
impl TypeRedefinition {
    pub fn redefine_type(&mut self, ty_id: TypeId, new_def: TypeDef) {
        // Increment type generation
        let gen = self.type_env.increment_generation();
        
        // Store new definition with generation
        self.type_env.redefine(ty_id, new_def, gen);
        
        // Invalidate all dependent compiled code
        let deps = self.dep_graph.type_dependents(ty_id);
        for dep in deps {
            self.cache.invalidate(dep);
        }
        
        // For safety: if >50% cache invalidated, clear all
        if deps.len() > self.cache.size() / 2 {
            self.cache.clear();
            println!("Note: Major type change, cleared compilation cache");
        }
    }
}
```

Types can change during redefinition:

```
ruchy> struct Point { x: i32, y: i32 }
ruchy> let p = Point { x: 1, y: 2 }
ruchy> struct Point { x: f64, y: f64, z: f64 }  // Redefine
Note: Type 'Point' redefined, 3 dependent functions recompiled
ruchy> let p2 = Point { x: 1.0, y: 2.0, z: 3.0 }  // New layout
```

## 10. Security Model

### 10.1 Trust Levels

```rust
enum TrustLevel {
    Sandboxed,   // Default: restricted capabilities
    Trusted,     // User-approved: full system access
    Untrusted,   // Explicit: from external sources
}

struct SecurityContext {
    trust_level: TrustLevel,
    allowed_crates: HashSet<String>,
    shell_enabled: bool,
    network_enabled: bool,
}
```

### 10.2 Security Policies

```rust
impl SecurityContext {
    pub fn check_shell_access(&self) -> Result<()> {
        match self.trust_level {
            TrustLevel::Trusted => Ok(()),
            _ => Err(SecurityError::ShellAccessDenied)
        }
    }
    
    pub fn verify_crate_allowed(&self, crate_name: &str) -> Result<()> {
        if self.allowed_crates.contains(crate_name) {
            return Ok(());
        }
        
        // Prompt user for first-time crates
        if self.prompt_user_approval(crate_name)? {
            self.allowed_crates.insert(crate_name.to_string());
            Ok(())
        } else {
            Err(SecurityError::CrateNotAllowed)
        }
    }
}
```

Configuration in `~/.ruchy/security.toml`:
```toml
[security]
default_trust = "sandboxed"
auto_approve_std_crates = true
shell_requires_confirmation = true

[allowed_crates]
# Pre-approved crates
tokio = true
serde = true
```

## 11. Error Recovery

### 11.1 Parse Recovery

```rust
impl Parser {
    fn parse_repl(&mut self, input: &str) -> ParseResult {
        match self.parse_complete(input) {
            Ok(ast) => Ok(ast),
            Err(Incomplete) => Err(NeedMoreInput),
            Err(SyntaxError(e)) => {
                // Attempt recovery
                let partial = self.parse_until_error(input);
                Err(PartialParse(partial, e))
            }
        }
    }
}
```

### 11.2 Panic Isolation

REPL continues after panics:

```rust
impl Executor {
    fn execute_isolated(&mut self, code: Code) -> Result<Value> {
        std::panic::catch_unwind(|| {
            self.execute_inner(code)
        }).unwrap_or_else(|panic| {
            Err(ExecutionError::Panic(panic))
        })
    }
}
```

## 12. Performance Requirements

### 12.1 Latency Targets

- First keystroke to response: <10ms
- Simple expression evaluation: <15ms  
- Module import (cached): <50ms
- Module import (first): <500ms
- JIT compilation trigger: <100ms

### 12.2 Memory Constraints

- Base memory: <50MB
- Per-session growth: <1MB/1000 commands
- Cache size limit: 500MB (configurable)

## 13. Configuration

### 13.1 User Configuration

`~/.ruchy/repl.toml`:
```toml
[performance]
jit_threshold = 100
cache_size_mb = 500

[display]
max_width = 120
color_scheme = "monokai"

[history]
max_entries = 10000
sync_enabled = true

[security]
trust_mode = "sandboxed"  # or "trusted"
```

### 13.2 Project Configuration

`.ruchy-repl.toml`:
```toml
[startup]
prelude = ["use std::collections::*", "use tokio::prelude::*"]
exec = "scripts/repl_init.ruchy"
trust_required = true  # Requires --trust flag to execute

[environment]
features = ["dataframe", "plotting"]
```

## 14. First-Run Experience

### 14.1 Onboarding Flow

On first launch, the REPL:

1. Creates default directories (`~/.ruchy/cache`, etc.)
2. Generates minimal config with wizard:
   ```
   Welcome to Ruchy REPL v1.0
   
   Quick setup (Y to accept defaults, N to customize):
   - Enable syntax highlighting? [Y/n]
   - Enable history? [Y/n]
   - Trust level (sandboxed/trusted)? [sandboxed]
   
   Configuration saved to ~/.ruchy/repl.toml
   Type ? for help, or start coding!
   ```

3. Shows mode indicators:
   ```
   ruchy> ? intro
   Available modes:
   - ] for package management
   - ; for shell commands (requires trust)
   - @ for AI assistance
   - / for history search
   ```

## 15. Testing Strategy

### 15.1 Property Tests

```rust
quickcheck! {
    fn repl_deterministic(cmds: Vec<Command>) -> bool {
        let result1 = run_repl_session(&cmds);
        let result2 = run_repl_session(&cmds);
        result1 == result2
    }
    
    fn cache_transparent(expr: Expr) -> bool {
        let fresh = eval_fresh(expr);
        let cached = eval_with_cache(expr);
        fresh == cached
    }
    
    fn type_redefinition_safe(old: TypeDef, new: TypeDef) -> bool {
        let session = setup_session();
        session.define_type("T", old);
        let v1 = session.eval("T::new()");
        session.redefine_type("T", new);
        let v2 = session.eval("T::new()");
        
        // Old and new values coexist safely
        v1.is_valid() && v2.is_valid()
    }
}
```

### 15.2 Benchmarks

Critical path benchmarks with regression detection:

```rust
#[bench]
fn bench_simple_eval(b: &mut Bencher) {
    let repl = setup_repl();
    b.iter(|| {
        repl.eval("2 + 2")
    });
    assert!(b.ns_per_iter() < 15_000_000); // 15ms
}

#[bench]
fn bench_type_redefine(b: &mut Bencher) {
    let repl = setup_repl();
    repl.eval("struct S { x: i32 }");
    b.iter(|| {
        repl.eval("struct S { x: i32, y: i32 }")
    });
    assert!(b.ns_per_iter() < 100_000_000); // 100ms
}
```

## 16. Implementation Phases

**Phase 1 (Q1 2025):** Core interpreter, basic state management  
**Phase 2 (Q2 2025):** JIT compilation, module imports, security framework  
**Phase 3 (Q3 2025):** Full introspection, async support, type redefinition  
**Phase 4 (Q4 2025):** MCP integration, advanced display, onboarding

## 17. Non-Goals

- Remote REPL sessions (use LSP instead)
- Notebook interface (separate project)
- Time-travel debugging (future consideration)
- Multi-user sessions (security complexity)
- Cross-platform GUI (terminal-only focus)

## 18. Appendix: Command Reference

### 18.1 Mode Commands

| Mode | Trigger | Example | Description |
|------|---------|---------|-------------|
| Ruchy | (default) | `let x = 42` | Execute Ruchy code |
| Cargo | `]` | `] add serde` | Package management |
| Shell | `;` | `; ls -la` | System commands (requires trust) |
| Help | `?` | `? Vec::push` | Documentation lookup |
| MCP | `@` | `@ explain this error` | AI assistance (async) |
| Search | `/` | `/ http request` | History search |

### 18.2 Introspection Commands

| Command | Example | Output |
|---------|---------|--------|
| `:t` | `:t vec![1,2,3]` | `Vec<i32>` |
| `?` | `HashMap?` | Shows documentation |
| `??` | `my_func??` | Shows source code |
| `???` | `compute???` | Shows MIR/assembly |
| `:i` | `:i Iterator` | Lists implementations |

---

This specification defines a REPL that achieves Python's interactivity with Rust's performance guarantees, serving as the foundation for Ruchy's interactive development experience.