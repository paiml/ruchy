# Ruchy WASM REPL Specification

## Overview

The WASM REPL provides browser-based interactive evaluation of Ruchy code through progressive enhancement: parse → typecheck → interpret → compile. This specification defines the architecture, implementation phases, and deployment strategy for both the development REPL and production script compilation.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser Environment                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   HTML UI   │←→│ WASM Module  │←→│ SharedArrayBuffer│  │
│  │  (50 LOC)   │  │   (~200KB)   │  │   (DataFrames)   │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
│         ↓                ↓                     ↓            │
│  ┌─────────────────────────────────────────────────────┐  │
│  │            Web Workers (for heavy compute)           │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Memory Model

- **Linear Memory**: Single WebAssembly.Memory (initially 4MB, max 64MB)
- **Stack**: 256KB for recursion (configurable via `__wasm_stack_size`)
- **Heap**: Arena allocation with generational collection
- **GC Strategy**: Bump allocation with mark-compact for old generation

```rust
pub struct ReplHeap {
    young: BumpArena<256_KB>,  // Fast allocation, reset between evaluations
    old: BumpArena<2_MB>,       // Long-lived bindings, compacted periodically
    roots: Vec<ValueRef>,       // GC roots from environment
}

impl ReplHeap {
    pub fn minor_gc(&mut self) {
        // Just reset young generation pointer
        self.young.reset();
    }
    
    pub fn major_gc(&mut self) {
        // Mark from roots, compact old generation
        let live = self.mark_from_roots();
        self.old.compact(live);
    }
}
```

## Implementation Phases

### Phase 1: Type-Checking REPL (Week 1)

#### Cargo Configuration

```toml
# crates/ruchy-wasm/Cargo.toml
[package]
name = "ruchy-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
ruchy-parser = { path = "../ruchy-parser" }
ruchy-typeck = { path = "../ruchy-typeck" }
console_error_panic_hook = "0.1"
wee_alloc = "0.4"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols
panic = "abort"     # No unwinding

[package.metadata.wasm-pack]
"wasm-opt" = ["-Oz", "--enable-simd"]
```

#### Core Implementation

```rust
// crates/ruchy-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use ruchy_parser::{Parser, ParseError};
use ruchy_typeck::{TypeChecker, Type, TypeError};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

#[derive(Serialize, Deserialize)]
pub struct ReplOutput {
    success: bool,
    display: Option<String>,
    type_info: Option<String>,
    rust_code: Option<String>,
    error: Option<String>,
    timing: TimingInfo,
}

#[derive(Serialize, Deserialize)]
pub struct TimingInfo {
    parse_ms: f64,
    typecheck_ms: f64,
    total_ms: f64,
}

#[wasm_bindgen]
pub struct RuchyRepl {
    parser: Parser,
    checker: TypeChecker,
    bindings: HashMap<String, Type>,
    history: Vec<String>,
}

#[wasm_bindgen]
impl RuchyRepl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RuchyRepl, JsValue> {
        console_error_panic_hook::set_once();
        
        Ok(RuchyRepl {
            parser: Parser::new(),
            checker: TypeChecker::new(),
            bindings: HashMap::new(),
            history: Vec::new(),
        })
    }
    
    pub fn eval(&mut self, input: &str) -> Result<JsValue, JsValue> {
        let start = performance::now();
        
        // Parse
        let parse_start = performance::now();
        let ast = match self.parser.parse(input) {
            Ok(ast) => ast,
            Err(e) => return Ok(serde_wasm_bindgen::to_value(&ReplOutput {
                success: false,
                display: None,
                type_info: None,
                rust_code: None,
                error: Some(format!("Parse error: {}", e)),
                timing: TimingInfo {
                    parse_ms: performance::now() - parse_start,
                    typecheck_ms: 0.0,
                    total_ms: performance::now() - start,
                },
            })?),
        };
        let parse_time = performance::now() - parse_start;
        
        // Typecheck
        let typecheck_start = performance::now();
        let typed = match self.checker.infer(&ast, &self.bindings) {
            Ok(typed) => typed,
            Err(e) => return Ok(serde_wasm_bindgen::to_value(&ReplOutput {
                success: false,
                display: Some(ast.to_string()),
                type_info: None,
                rust_code: None,
                error: Some(format!("Type error: {}", e)),
                timing: TimingInfo {
                    parse_ms: parse_time,
                    typecheck_ms: performance::now() - typecheck_start,
                    total_ms: performance::now() - start,
                },
            })?),
        };
        let typecheck_time = performance::now() - typecheck_start;
        
        // Update bindings for future evaluations
        if let Some(binding) = extract_binding(&ast) {
            self.bindings.insert(binding.name, binding.ty);
        }
        
        // Generate Rust code
        let rust_code = codegen::to_rust(&typed);
        
        // Store in history
        self.history.push(input.to_string());
        
        Ok(serde_wasm_bindgen::to_value(&ReplOutput {
            success: true,
            display: Some(ast.to_string()),
            type_info: Some(typed.ty.to_string()),
            rust_code: Some(rust_code),
            error: None,
            timing: TimingInfo {
                parse_ms: parse_time,
                typecheck_ms: typecheck_time,
                total_ms: performance::now() - start,
            },
        })?)
    }
    
    pub fn get_completions(&self, partial: &str) -> Vec<JsValue> {
        self.bindings
            .iter()
            .filter(|(name, _)| name.starts_with(partial))
            .map(|(name, ty)| {
                JsValue::from_str(&format!("{}: {}", name, ty))
            })
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.history.clear();
    }
    
    pub fn get_history(&self) -> Vec<JsValue> {
        self.history.iter()
            .map(|s| JsValue::from_str(s))
            .collect()
    }
}
```

## Unified Interpreter Architecture

```rust
// crates/ruchy-core/src/value.rs (shared between native and WASM)
#[cfg_attr(target_arch = "wasm32", derive(wasm_bindgen))]
#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    #[cfg(not(target_arch = "wasm32"))]
    List(Vec<Value>),
    #[cfg(target_arch = "wasm32")]
    List(Box<[Value]>),  // Fixed size for WASM
}

// Single interpreter implementation with platform-specific allocators
pub struct Interpreter<A: Allocator = DefaultAllocator> {
    heap: A,
    globals: HashMap<String, Value>,
}

#[cfg(not(target_arch = "wasm32"))]
type DefaultAllocator = SystemAllocator;

#[cfg(target_arch = "wasm32")]
type DefaultAllocator = ArenaAllocator;

// Macro generates both native and WASM evaluation from single source
macro_rules! define_eval {
    ($($pattern:pat => $body:expr),*) => {
        pub fn eval(&mut self, expr: &TypedExpr) -> Result<Value, String> {
            match expr {
                $($pattern => $body,)*
            }
        }
    };
}

impl<A: Allocator> Interpreter<A> {
    define_eval! {
        TypedExpr::Literal(lit) => self.eval_literal(lit),
        TypedExpr::Variable(name) => self.globals.get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined variable: {}", name)),
        TypedExpr::Let(name, value, body) => {
            let val = self.eval(value)?;
            self.globals.insert(name.clone(), val);
            self.eval(body)
        },
        TypedExpr::If(cond, then_expr, else_expr) => {
            match self.eval(cond)? {
                Value::Bool(true) => self.eval(then_expr),
                Value::Bool(false) => self.eval(else_expr),
                _ => Err("Type error: condition must be boolean".to_string())
            }
        }
    }
}
```

### Phase 3: Direct WAT Generation (Week 4+)

```rust
// crates/ruchy-wasm/src/wat_codegen.rs
use std::collections::HashMap;

pub struct WatGenerator {
    locals: HashMap<String, u32>,
    next_local: u32,
}

impl WatGenerator {
    pub fn compile_function(&mut self, name: &str, ast: &TypedExpr) -> String {
        // Direct AST → WAT without wasmtime dependency
        match ast {
            TypedExpr::Function(params, body) => {
                let param_list = params.iter()
                    .enumerate()
                    .map(|(i, p)| {
                        self.locals.insert(p.clone(), i as u32);
                        format!("(param ${} i32)", p)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                
                format!(
                    "(module
                      (func ${} {} (result {})
                        {}
                      )
                      (export \"{}\" (func ${}))
                    )",
                    name,
                    param_list,
                    self.wat_type(&body.ty),
                    self.compile_expr(body),
                    name,
                    name
                )
            },
            _ => panic!("Expected function")
        }
    }
    
    fn compile_expr(&mut self, expr: &TypedExpr) -> String {
        match expr {
            TypedExpr::Literal(Literal::Int(n)) => format!("i32.const {}", n),
            TypedExpr::Variable(name) => format!("local.get ${}", self.locals[name]),
            TypedExpr::BinOp(op, left, right) => format!(
                "{} {} {}",
                self.compile_expr(left),
                self.compile_expr(right),
                match op {
                    BinOp::Add => "i32.add",
                    BinOp::Sub => "i32.sub",
                    BinOp::Mul => "i32.mul",
                    BinOp::Div => "i32.div_s",
                    _ => unimplemented!(),
                }
            ),
            _ => unimplemented!(),
        }
    }
    
    fn wat_type(&self, ty: &Type) -> &'static str {
        match ty {
            Type::Int => "i32",
            Type::Float => "f64",
            Type::Bool => "i32",
            _ => "i32", // Default to i32 for complex types
        }
    }
}

// Dynamic instantiation in browser
#[wasm_bindgen]
impl RuchyRepl {
    pub async fn compile_hot(&mut self, name: &str, code: &str) -> Result<JsValue, JsValue> {
        let ast = self.parser.parse(code)?;
        let typed = self.checker.infer(&ast, &self.bindings)?;
        
        let mut generator = WatGenerator::new();
        let wat = generator.compile_function(name, &typed);
        
        // Return WAT for browser to compile via WebAssembly.instantiate()
        Ok(JsValue::from_str(&wat))
    }
}
```

## Progressive Loading Strategy

```javascript
// Core modules load on-demand via ES module imports
const core = await import('./ruchy_core.wasm');     // 80KB - Parser + Typechecker
const interp = () => import('./ruchy_interp.wasm');  // 120KB - Lazy load interpreter  
const df = () => import('./ruchy_df.wasm');          // 50KB - DataFrame support

class RuchyReplLoader {
    constructor() {
        this.core = null;
        this.interpreter = null;
        this.dataframe = null;
    }
    
    async init() {
        // Load only core initially
        this.core = await core;
        return this.core.RuchyRepl.new();
    }
    
    async ensureInterpreter() {
        if (!this.interpreter) {
            this.interpreter = await interp();
        }
        return this.interpreter;
    }
    
    async ensureDataFrame() {
        if (!this.dataframe) {
            this.dataframe = await df();
        }
        return this.dataframe;
    }
}
```

## I/O Capability Bridge

```rust
// crates/ruchy-wasm/src/io_bridge.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct IoCapabilities {
    read_file: Option<js_sys::Function>,
    http_get: Option<js_sys::Function>,
    write_output: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl RuchyRepl {
    pub fn set_io_capabilities(&mut self, caps: JsValue) {
        // JavaScript provides sandboxed I/O implementations
        let obj = js_sys::Object::from(caps);
        self.io = IoCapabilities {
            read_file: js_sys::Reflect::get(&obj, &"read_file".into())
                .ok()
                .and_then(|v| v.dyn_into::<js_sys::Function>().ok()),
            http_get: js_sys::Reflect::get(&obj, &"http_get".into())
                .ok()
                .and_then(|v| v.dyn_into::<js_sys::Function>().ok()),
            write_output: js_sys::Reflect::get(&obj, &"write_output".into())
                .ok()
                .and_then(|v| v.dyn_into::<js_sys::Function>().ok()),
        };
    }
    
    async fn read_file(&self, path: &str) -> Result<String, JsValue> {
        if let Some(ref f) = self.io.read_file {
            let promise = f.call1(&JsValue::NULL, &JsValue::from_str(path))?;
            let result = wasm_bindgen_futures::JsFuture::from(promise.into()).await?;
            Ok(result.as_string().unwrap_or_default())
        } else {
            Err(JsValue::from_str("File I/O not available in WASM context"))
        }
    }
}
```

JavaScript configuration:
```javascript
// Provide controlled I/O access
repl.set_io_capabilities({
    read_file: async (path) => {
        // Sandbox to examples directory
        if (!path.startsWith('/examples/')) {
            throw new Error('Access denied');
        }
        return await fetch(path).then(r => r.text());
    },
    http_get: async (url) => {
        // Require HTTPS, no local network access
        const u = new URL(url);
        if (u.protocol !== 'https:') {
            throw new Error('HTTPS required');
        }
        return await fetch(url).then(r => r.json());
    },
    write_output: (text) => {
        console.log(text);
        outputDiv.appendChild(createOutputLine(text));
    }
});
```

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ruchy REPL</title>
    <style>
        :root {
            --bg: #0d1117;
            --bg-secondary: #161b22;
            --text: #c9d1d9;
            --text-secondary: #8b949e;
            --accent: #58a6ff;
            --error: #f85149;
            --success: #3fb950;
            --border: #30363d;
        }
        
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
            background: var(--bg);
            color: var(--text);
            height: 100vh;
            display: grid;
            grid-template-rows: auto 1fr auto;
        }
        
        header {
            background: var(--bg-secondary);
            padding: 1rem;
            border-bottom: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        #output {
            overflow-y: auto;
            padding: 1rem;
        }
        
        .entry {
            margin-bottom: 1rem;
        }
        
        .input-line {
            color: var(--text-secondary);
            margin-bottom: 0.25rem;
        }
        
        .input-line::before {
            content: '> ';
            color: var(--accent);
        }
        
        .result {
            padding-left: 1rem;
            border-left: 2px solid var(--accent);
        }
        
        .type-info {
            color: var(--text-secondary);
            font-size: 0.9em;
            margin-top: 0.25rem;
        }
        
        .error {
            color: var(--error);
            padding-left: 1rem;
            border-left: 2px solid var(--error);
        }
        
        .timing {
            color: var(--text-secondary);
            font-size: 0.8em;
            margin-top: 0.5rem;
        }
        
        #input-container {
            background: var(--bg-secondary);
            border-top: 1px solid var(--border);
            padding: 1rem;
            display: flex;
            gap: 0.5rem;
        }
        
        #input {
            flex: 1;
            background: var(--bg);
            color: var(--text);
            border: 1px solid var(--border);
            padding: 0.5rem;
            font-family: inherit;
            font-size: 14px;
            outline: none;
        }
        
        #input:focus {
            border-color: var(--accent);
        }
        
        button {
            background: var(--accent);
            color: var(--bg);
            border: none;
            padding: 0.5rem 1rem;
            cursor: pointer;
            font-family: inherit;
            font-weight: 600;
        }
        
        button:hover {
            opacity: 0.9;
        }
        
        .loading {
            text-align: center;
            padding: 2rem;
            color: var(--text-secondary);
        }
        
        .spinner {
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 2px solid var(--border);
            border-top-color: var(--accent);
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
        }
        
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        
        code {
            background: var(--bg);
            padding: 0.125rem 0.25rem;
            border-radius: 3px;
        }
        
        .rust-preview {
            background: var(--bg);
            padding: 0.5rem;
            margin-top: 0.5rem;
            border: 1px solid var(--border);
            font-size: 0.9em;
            display: none;
        }
        
        .rust-preview.show {
            display: block;
        }
    </style>
</head>
<body>
    <header>
        <h1>Ruchy REPL</h1>
        <div class="status">
            <span id="version">Loading...</span>
        </div>
    </header>
    
    <div id="output">
        <div class="loading">
            <div class="spinner"></div>
            <p>Loading WASM module...</p>
        </div>
    </div>
    
    <div id="input-container">
        <input 
            id="input" 
            type="text" 
            placeholder="Enter Ruchy code (e.g., let x = 42)" 
            disabled
            autocomplete="off"
            spellcheck="false"
        >
        <button id="run" disabled>Run</button>
        <button id="clear">Clear</button>
    </div>
    
    <script type="module">
        import init, { RuchyRepl } from './ruchy_wasm.js';
        
        let repl = null;
        const output = document.getElementById('output');
        const input = document.getElementById('input');
        const runBtn = document.getElementById('run');
        const clearBtn = document.getElementById('clear');
        const versionSpan = document.getElementById('version');
        
        // Initialize WASM
        async function initRepl() {
            try {
                await init();
                repl = new RuchyRepl();
                
                // Clear loading message
                output.innerHTML = '';
                
                // Enable input
                input.disabled = false;
                runBtn.disabled = false;
                input.focus();
                
                // Set version
                versionSpan.textContent = 'v0.1.0';
                
                // Add welcome message
                addOutput('Welcome to Ruchy REPL! Type expressions to evaluate them.', 'info');
                
            } catch (error) {
                output.innerHTML = `<div class="error">Failed to initialize: ${error}</div>`;
            }
        }
        
        function addOutput(text, className = '') {
            const div = document.createElement('div');
            div.className = className;
            div.textContent = text;
            output.appendChild(div);
            output.scrollTop = output.scrollHeight;
        }
        
        function addEntry(inputText, result) {
            const entry = document.createElement('div');
            entry.className = 'entry';
            
            // Input line
            const inputLine = document.createElement('div');
            inputLine.className = 'input-line';
            inputLine.textContent = inputText;
            entry.appendChild(inputLine);
            
            if (result.success) {
                // Result
                if (result.display) {
                    const resultDiv = document.createElement('div');
                    resultDiv.className = 'result';
                    resultDiv.textContent = result.display;
                    entry.appendChild(resultDiv);
                }
                
                // Type info
                if (result.type_info) {
                    const typeDiv = document.createElement('div');
                    typeDiv.className = 'type-info';
                    typeDiv.textContent = `: ${result.type_info}`;
                    entry.appendChild(typeDiv);
                }
                
                // Rust preview (collapsible)
                if (result.rust_code) {
                    const rustDiv = document.createElement('div');
                    rustDiv.className = 'rust-preview';
                    rustDiv.innerHTML = `<strong>Rust:</strong><br><pre>${escapeHtml(result.rust_code)}</pre>`;
                    entry.appendChild(rustDiv);
                }
            } else {
                // Error
                const errorDiv = document.createElement('div');
                errorDiv.className = 'error';
                errorDiv.textContent = result.error || 'Unknown error';
                entry.appendChild(errorDiv);
            }
            
            // Timing
            if (result.timing) {
                const timingDiv = document.createElement('div');
                timingDiv.className = 'timing';
                timingDiv.textContent = `Parse: ${result.timing.parse_ms.toFixed(2)}ms | ` +
                                      `Typecheck: ${result.timing.typecheck_ms.toFixed(2)}ms | ` +
                                      `Total: ${result.timing.total_ms.toFixed(2)}ms`;
                entry.appendChild(timingDiv);
            }
            
            output.appendChild(entry);
            output.scrollTop = output.scrollHeight;
        }
        
        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
        
        async function runCode() {
            const code = input.value.trim();
            if (!code || !repl) return;
            
            try {
                const result = await repl.eval(code);
                addEntry(code, result);
                input.value = '';
                
                // Store in history
                if (window.localStorage) {
                    const history = JSON.parse(localStorage.getItem('ruchy_history') || '[]');
                    history.push(code);
                    if (history.length > 100) history.shift();
                    localStorage.setItem('ruchy_history', JSON.stringify(history));
                }
            } catch (error) {
                addEntry(code, {
                    success: false,
                    error: error.toString()
                });
            }
        }
        
        // Event handlers
        runBtn.addEventListener('click', runCode);
        
        input.addEventListener('keydown', async (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                await runCode();
            }
            
            // History navigation
            if (e.key === 'ArrowUp') {
                // TODO: Implement history navigation
            }
            
            // Tab completion
            if (e.key === 'Tab') {
                e.preventDefault();
                if (repl) {
                    const partial = input.value.split(/\s+/).pop();
                    const completions = repl.get_completions(partial);
                    if (completions.length === 1) {
                        // Auto-complete
                        const parts = input.value.split(/\s+/);
                        parts[parts.length - 1] = completions[0].split(':')[0];
                        input.value = parts.join(' ');
                    } else if (completions.length > 1) {
                        // Show options
                        addOutput(`Completions: ${completions.join(', ')}`, 'info');
                    }
                }
            }
        });
        
        clearBtn.addEventListener('click', () => {
            output.innerHTML = '';
            if (repl) {
                repl.clear();
                addOutput('Environment cleared', 'info');
            }
        });
        
        // Initialize on load
        initRepl();
    </script>
</body>
</html>
```

## Repository Structure

```
ruchy/
├── crates/
│   ├── ruchy-core/        # Shared AST, types, interpreter (no_std)
│   ├── ruchy-parser/      # Parser (no_std compatible)
│   ├── ruchy-typeck/      # Type inference engine (no_std)
│   ├── ruchy-native/      # Native runtime with std
│   ├── ruchy-wasm/        # WASM bindings and bridge
│   └── ruchy-web/         # HTML/JS assets
├── tests/
│   └── conformance/       # Shared test corpus for native + WASM
├── examples/              # Example programs (test both targets)
└── .github/
    └── workflows/
        ├── test.yml       # Native + WASM tests on every PR
        └── deploy.yml     # Deploy to GitHub Pages on merge

### Local Development

```bash
#!/bin/bash
# scripts/build-wasm.sh

set -e

echo "Building WASM module..."
cd crates/ruchy-wasm

# Install dependencies if needed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build WASM module
wasm-pack build --target web --release

# Optimize with wasm-opt
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM size..."
    wasm-opt -Oz \
        --enable-simd \
        --enable-bulk-memory \
        --converge \
        pkg/ruchy_wasm_bg.wasm \
        -o pkg/ruchy_wasm_bg.wasm
fi

# Report size
echo "WASM module size:"
ls -lh pkg/ruchy_wasm_bg.wasm

# Copy to docs for GitHub Pages
mkdir -p ../../docs/playground
cp pkg/* ../../docs/playground/
cp ../../crates/ruchy-repl-web/index.html ../../docs/playground/

echo "Build complete! Open docs/playground/index.html to test locally"
```

### GitHub Actions CI/CD

```yaml
# .github/workflows/deploy-repl.yml
name: Deploy WASM REPL

on:
  push:
    branches: [main]
    paths:
      - 'crates/ruchy-parser/**'
      - 'crates/ruchy-typeck/**'
      - 'crates/ruchy-wasm/**'
      - 'crates/ruchy-repl-web/**'
      - '.github/workflows/deploy-repl.yml'

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: crates/ruchy-wasm
      
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      
      - name: Install wasm-opt
        run: |
          npm install -g wasm-opt
      
      - name: Build WASM module
        run: |
          cd crates/ruchy-wasm
          wasm-pack build --target web --release
          
      - name: Optimize WASM
        run: |
          wasm-opt -Oz \
            --enable-simd \
            --converge \
            crates/ruchy-wasm/pkg/ruchy_wasm_bg.wasm \
            -o crates/ruchy-wasm/pkg/ruchy_wasm_bg.wasm
      
      - name: Prepare deployment
        run: |
          mkdir -p docs/playground
          cp crates/ruchy-wasm/pkg/* docs/playground/
          cp crates/ruchy-repl-web/index.html docs/playground/
          echo "ruchy-lang.org" > docs/CNAME
      
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs
          force_orphan: true
```

## Performance Targets

### Size Budget
- Core (parser + typechecker): <80KB gzipped
- Interpreter module: <120KB gzipped  
- DataFrame module: <50KB gzipped
- Total with all modules: <250KB gzipped

### Speed Targets
- Initial core load: <200ms on 3G
- First evaluation: <20ms
- Type checking: <5ms for 100 LOC
- Completion suggestions: <2ms

### Memory Usage
- Base heap: <4MB
- Young generation arena: 256KB
- Old generation arena: 2MB
- Per DataFrame: Direct linear memory mapping

## Testing Strategy

## Conformance Testing Strategy

```rust
// tests/conformance/runner.rs
#[cfg(test)]
mod conformance {
    use ruchy_core::*;
    
    macro_rules! test_both_targets {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                // Native execution
                let native_result = ruchy_native::eval($input);
                assert_eq!(native_result, $expected);
                
                // WASM execution
                #[cfg(target_arch = "wasm32")]
                {
                    let wasm_result = ruchy_wasm::eval($input);
                    assert_eq!(wasm_result, $expected);
                }
            }
        };
    }
    
    test_both_targets!(arithmetic, "1 + 2 * 3", Value::Int(7));
    test_both_targets!(let_binding, "let x = 5; x + 1", Value::Int(6));
    test_both_targets!(function, "let f = |x| x * 2; f(21)", Value::Int(42));
    
    // Property tests ensure semantic equivalence
    proptest! {
        #[test]
        fn native_wasm_equivalence(input in valid_expr()) {
            let native = ruchy_native::eval(&input);
            let wasm = ruchy_wasm::eval(&input);
            prop_assert_eq!(native, wasm);
        }
    }
}
```

GitHub Actions enforcement:
```yaml
# .github/workflows/test.yml
name: Conformance Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Test Native
        run: cargo test --workspace
      
      - name: Test WASM
        run: |
          cargo install wasm-pack
          wasm-pack test --headless --chrome crates/ruchy-wasm
      
      - name: Conformance
        run: |
          cargo test --test conformance
          wasm-pack test --headless --chrome tests/conformance
```

## Individual Script Compilation

### Script → WASM Compilation

```rust
// crates/ruchy-cli/src/wasm_target.rs
pub fn compile_to_wasm(input_path: &Path, output_path: &Path) -> Result<()> {
    // Parse and typecheck
    let source = fs::read_to_string(input_path)?;
    let ast = parse(&source)?;
    let typed = typecheck(&ast)?;
    
    // Generate Rust code with no_std
    let rust_code = generate_wasm_rust(&typed)?;
    
    // Write temporary Rust file
    let temp_dir = tempdir()?;
    let rust_path = temp_dir.path().join("lib.rs");
    fs::write(&rust_path, rust_code)?;
    
    // Create minimal Cargo.toml
    let cargo_toml = r#"
        [package]
        name = "ruchy_script"
        version = "0.1.0"
        edition = "2021"
        
        [lib]
        crate-type = ["cdylib"]
        
        [dependencies]
        wasm-bindgen = "0.2"
        
        [profile.release]
        opt-level = "z"
        lto = true
        panic = "abort"
    "#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)?;
    
    // Compile with wasm-pack
    Command::new("wasm-pack")
        .args(&["build", "--target", "web", "--release"])
        .current_dir(&temp_dir)
        .status()?;
    
    // Copy output
    fs::copy(
        temp_dir.path().join("pkg").join("ruchy_script_bg.wasm"),
        output_path
    )?;
    
    Ok(())
}
```

### Generated TypeScript Bindings

```typescript
// analytics.d.ts (auto-generated)
export interface DataFrame {
    columns: string[];
    data: Float64Array;
    shape: [number, number];
}

export function process(data: Float64Array): DataFrame;
export function detect_anomalies(df: DataFrame, threshold: number): number[];
```

## Deployment Checklist

- [ ] WASM module builds successfully
- [ ] Size is under 200KB gzipped
- [ ] All example programs evaluate correctly
- [ ] Type errors display properly
- [ ] GitHub Actions deploys to Pages
- [ ] Performance metrics meet targets
- [ ] Mobile browser compatibility verified
- [ ] Offline mode works after first load
- [ ] Console has no errors in production

## Future Enhancements

### Phase 4: Advanced Features
- WebWorker support for parallel evaluation
- SharedArrayBuffer for zero-copy DataFrames
- IndexedDB for persistent REPL sessions
- WebGPU backend for numeric computations

### Phase 5: Production Features
- Source maps for debugging
- Hot module reloading during development
- WASM SIMD for vectorized operations
- Component model for modular loading

## Security Considerations

- **CSP Headers**: Require `'wasm-unsafe-eval'` for WebAssembly
- **Memory Limits**: Enforce maximum heap size (256MB default)
- **Execution Timeout**: Kill long-running evaluations (5s default)
- **Input Sanitization**: Prevent XSS via output escaping
- **No Network Access**: WASM runs in sandboxed environment

## Browser Compatibility

### Required Features
- WebAssembly 1.0 (all modern browsers)
- BigInt for i64 support (Chrome 67+, Firefox 68+)
- TextEncoder/TextDecoder (all modern browsers)

### Optional Enhancements
- WebAssembly SIMD (Chrome 91+, Firefox 89+)
- SharedArrayBuffer (requires COOP/COEP headers)
- WebWorkers (all modern browsers)

## Performance Monitoring

```javascript
// Telemetry collection (optional, privacy-respecting)
if (window.performance && window.performance.measure) {
    performance.mark('repl-start');
    await repl.eval(code);
    performance.mark('repl-end');
    performance.measure('repl-eval', 'repl-start', 'repl-end');
    
    const measure = performance.getEntriesByName('repl-eval')[0];
    console.log(`Evaluation took ${measure.duration}ms`);
}
```