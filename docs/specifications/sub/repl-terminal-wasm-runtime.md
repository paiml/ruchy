# Sub-spec: REPL Magic — Terminal REPL & WASM Runtime Architecture

**Parent:** [repl-magic-spec.md](../repl-magic-spec.md) Sections 5-7

---

- Direct DOM manipulation from Ruchy# Ruchy REPL Specification v3.0
## Interactive Distributed Data Science Platform

### Executive Summary

Ruchy implements a distributed REPL combining IPython/R/Julia ergonomics with actor-based concurrency and native MCP support. This specification defines the architecture, implementation phases, and performance requirements for a production-grade data science platform that transpiles to zero-cost Rust.

## Design Principles

**Core Invariants**
- Transpilation preserves semantics exactly
- Distribution requires no code changes
- Type safety extends through MCP boundaries
- Actor supervision guarantees fault recovery
- Session export produces production-ready code

**Non-negotiable Constraints**
- Pure Rust - no FFI to dynamic languages
- Terminal-first - GUI deferred
- Supervision trees mandatory for distributed operations
- Static typing including protocol schemas
- WASM-native notebook architecture

## Terminal REPL Implementation

### Phase 1: Core Mechanics (Months 1-3)

#### Extension Methods
```rust
// REPL syntax
df.head(5)
arr.mean()

// Transpiled form
use ruchy_std::DataFrameExt;
df.head(5)
```

Limited to 20 methods per type. Static dispatch only. Manual registration.

#### History Mechanism
```rust
>>> x = 42
>>> _ * 2      // Previous output
84
>>> _1         // Indexed history
42
```

Ring buffer implementation. 10,000 entry default. Persistent across sessions.

#### Introspection
```rust
?DataFrame     // Documentation
??DataFrame    // Source code
str(df)        // Structure
summary(df)    // Statistics
```

Leverages rustdoc. Source extracted via proc-macro.

#### Shell Integration
```rust
!ls -la        // Direct execution
let x = !pwd   // Capture output
```

Fork/exec model. Output captured via pipe.

#### Workspace
```rust
whos()                   // Variable listing
clear!(r"temp_.*")       // Regex removal  
save_image("ws.RData")   // Serialize state
```

MessagePack serialization. Incremental save supported.

### Phase 2: Advanced Features (Months 4-6)

#### Magic Commands
```rust
%time expr          // Single execution
%timeit expr        // Statistical (n=1000)
%run script.ruchy   // External execution
%debug              // Post-mortem
%profile expr       // Flamegraph generation
```

Implemented as AST transformations. No runtime cost.

#### Mode System
```rust
>>> x = 10          // Normal mode

pkg> add polars     // Package mode (])
shell> git status   // Shell mode (;)
help> DataFrame     // Help mode (?)
```

Modal parser. Backspace returns to normal.

#### Completion Engine
```rust
df.gro<TAB>
├── group_by()  // Groups DataFrame
└── grow()      // Expands capacity

\alpha<TAB> → α  // Unicode expansion
```

Trie-based lookup. Fuzzy matching via Levenshtein distance.

#### Session Export
```rust
// REPL session with exploratory code
>>> x = 10
>>> y = x * 2
>>> println(y)
>>> df = read_csv("data.csv")
>>> df.head()

// Exported as clean production script
fn main() -> Result<(), Error> {
    let x = 10;
    let y = x * 2;
    println(y);
    
    let df = read_csv("data.csv")?;
    // df.head() removed (display only)
    
    Ok(())
}
```

Export removes dead code, adds error handling, consolidates imports.

## WASM Runtime Architecture

### Browser Execution Model (Months 7-9)

#### Compilation Pipeline
```rust
// In-browser compilation chain
Source Code → AST → Type-checked AST → WASM Bytecode
                                         ↓
                                    Linear Memory
                                         ↓
                                    SharedArrayBuffer
```

Compiler size: 5MB gzipped. Cranelift backend for WASM generation.

#### Memory Model
```rust
// Zero-copy DataFrame sharing between cells
struct WasmDataFrame {
    // Stored in SharedArrayBuffer
    data: *mut u8,
    shape: (usize, usize),
    stride: usize,
}

impl WasmDataFrame {
    fn share_between_cells(&self) -> Handle {
        // Returns handle to shared memory region
        Handle::from_sab(self.data, self.shape)
    }
}
```

SharedArrayBuffer requires COOP/COEP headers. 4GB memory limit per notebook.

#### Cell Execution
```rust
#[wasm_bindgen]
pub struct NotebookCell {
    code: String,
    wasm_module: Option<Module>,
    memory: SharedMemory,
}

#[wasm_bindgen]
impl NotebookCell {
    pub async fn execute(&mut self) -> Result<JsValue> {
        // Compile to WASM on-demand
        let module = compile_ruchy(&self.code)?;
        
        // Execute with shared memory
        let result = module.instantiate(&self.memory)?;
        
        // Rich output rendering
        match result {
            Value::DataFrame(df) => render_table(df),
            Value::Plot(p) => render_canvas(p),
            _ => render_text(result)
        }
    }
}
```

#### WASM-Specific Optimizations
```rust
// SIMD operations for numerical computing
#[target_feature(enable = "simd128")]
fn vector_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    // Auto-vectorization via wasm-simd
}

// Bulk memory operations
#[target_feature(enable = "bulk-memory")]
fn copy_dataframe(src: &DataFrame) -> DataFrame {
    // Memory.copy instruction
}
```

SIMD provides 4x speedup for numerical operations. Bulk memory reduces copy overhead by 10x.

#### Service Worker Architecture
```javascript
// Background execution without blocking UI
self.addEventListener('message', async (e) => {
    const { code, cellId } = e.data;
    
    // Compile and execute in worker
    const module = await compileRuchy(code);
    const result = await module.run();
    
    // Post result back to main thread
    self.postMessage({ cellId, result });
});
```

Workers enable parallel cell execution. Maximum 4 concurrent workers.

#### Notebook Persistence
```rust
// IndexedDB storage with incremental saves
struct NotebookStorage {
    db: IdbDatabase,
    checkpoint_interval: Duration,
}

impl NotebookStorage {
    async fn save_incremental(&self, cells: &[Cell]) {
        // Only save modified cells
        let dirty = cells.iter().filter(|c| c.modified);
        
        // Transaction-based updates
        let tx = self.db.transaction(&["cells"], "readwrite");
        for cell in dirty {
            tx.put(&cell.id, &cell.serialize())?;
        }
    }
}
```

IndexedDB allows 50GB+ storage. Automatic checkpoint every 30 seconds.

### WASM/Native Parity

#### Export Format
```rust
// Notebook format (.ruchynb)
{
  "version": "1.0",
  "kernel": "wasm",
  "cells": [
    {
      "type": "code",
      "source": "df = read_csv('data.csv')",
      "wasm_features": ["simd128", "bulk-memory"],
      "execution_time_ms": 15
    }
  ]
}

// Exports to identical Ruchy script
ruchy export notebook.ruchynb --target native
```

Bidirectional compatibility guaranteed. No semantic differences between WASM and native.

#### Performance Characteristics

| Operation | Native | WASM | WASM+SIMD |
|-----------|--------|------|-----------|
| DataFrame scan | 1x | 2.5x | 1.5x |
| Matrix multiply | 1x | 3x | 1.2x |
| String operations | 1x | 2x | 2x |
| Memory allocation | 1x | 1.5x | 1.5x |

Acceptable performance degradation for browser convenience.
