# 🔧 Ruchy Notebook Developer Guide

**Version 1.90.0** - Complete developer documentation for extending and integrating Ruchy Notebook

---

## 🎯 Overview

Ruchy Notebook is built with a modular, extensible architecture designed for:
- **Performance**: WASM runtime with WebWorker execution
- **Compatibility**: Modern web standards and progressive enhancement
- **Extensibility**: Clean APIs for customization and integration
- **Maintainability**: TypeScript-ready with comprehensive testing

---

## 📖 Table of Contents

- [🏗️ Architecture Overview](#️-architecture-overview)
- [🚀 Quick Start Development](#-quick-start-development)
- [📚 API Reference](#-api-reference)
- [🔧 Configuration Options](#-configuration-options)
- [🔌 Plugin System](#-plugin-system)
- [🧪 Testing Framework](#-testing-framework)
- [🚀 Deployment Guide](#-deployment-guide)
- [🛠️ Advanced Integration](#️-advanced-integration)
- [📊 Performance Optimization](#-performance-optimization)
- [🐛 Debugging Guide](#-debugging-guide)

---

## 🏗️ Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────┐
│                Frontend Layer                   │
├─────────────────────────────────────────────────┤
│  RuchyNotebook  │  WebWorker    │  ServiceWorker │
│  (Main UI)      │  (Execution)  │  (Caching)     │
├─────────────────────────────────────────────────┤
│                WASM Layer                       │
├─────────────────────────────────────────────────┤
│  WasmNotebook   │  VirtualMachine │ Memory Mgmt │
│  (Bindings)     │  (Interpreter)  │ (Arena/Slab) │
├─────────────────────────────────────────────────┤
│                Core Layer                       │
├─────────────────────────────────────────────────┤
│  Compiler       │  Parser         │  Type System │
│  (AST→Bytecode) │  (Text→AST)     │ (Inference)  │
└─────────────────────────────────────────────────┘
```

### Key Design Principles

**1. Separation of Concerns**
- UI logic separated from execution logic
- WASM runtime isolated in WebWorkers
- Progressive enhancement for feature detection

**2. Performance First**
- <50ms cell execution target
- <200KB WASM bundle size
- Virtual scrolling for 1000+ cells
- Lazy loading and caching strategies

**3. Modern Web Standards**
- WebAssembly for performance-critical code
- Service Workers for offline functionality
- Progressive Web App capabilities
- Mobile-first responsive design

**4. Extensibility**
- Plugin architecture for custom features
- Event system for integration hooks
- Configurable theming and layout
- Custom output renderers

---

## 🚀 Quick Start Development

### Prerequisites

```bash
# Required tools
rustc 1.75+              # Rust compiler
wasm-pack 0.12+          # WASM build tool
node.js 18+              # JavaScript runtime
npm 9+                   # Package manager
```

### Development Setup

```bash
# Clone repository
git clone https://github.com/paiml/ruchy.git
cd ruchy/ruchy-notebook

# Build WASM module
wasm-pack build --target web --out-dir pkg --release --no-default-features --features wasm

# Start development server
python -m http.server 8000  # Or any static server
# Navigate to http://localhost:8000
```

### Project Structure

```
ruchy-notebook/
├── src/                    # Rust source code
│   ├── wasm/              # WASM bindings and exports
│   ├── vm/                # Virtual machine implementation
│   ├── memory/            # Memory management (arena/slab)
│   ├── error/             # Error handling and suggestions
│   └── converter/         # Demo-to-notebook conversion
├── js/                    # JavaScript frontend
│   ├── ruchy-notebook.js  # Main notebook class
│   ├── ruchy-worker.js    # WebWorker implementation
│   ├── sw.js              # Service Worker
│   └── manifest.json      # PWA manifest
├── docs/                  # Documentation
├── testing/               # Performance test suites
├── examples/              # Example notebooks
└── pkg/                   # Generated WASM output
```

### Basic Integration Example

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Ruchy Notebook</title>
</head>
<body>
    <div id="notebook-container"></div>
    
    <script type="module">
        import { RuchyNotebook } from './js/ruchy-notebook.js';
        
        const container = document.getElementById('notebook-container');
        const notebook = new RuchyNotebook(container, {
            theme: 'dark',
            autoSave: true,
            useWorker: true,
            maxCellCount: 50
        });
    </script>
</body>
</html>
```

---

## 📚 API Reference

### RuchyNotebook Class

#### Constructor

```javascript
const notebook = new RuchyNotebook(container, options);
```

**Parameters:**
- `container` (HTMLElement): DOM element to contain the notebook
- `options` (Object): Configuration options

**Options:**
```javascript
{
    theme: 'dark' | 'light',           // UI theme
    autoSave: boolean,                 // Auto-save enabled
    saveInterval: number,              // Auto-save interval (ms)
    maxCellCount: number,              // Maximum cells
    useWorker: boolean,                // Enable WebWorker execution
    lazyLoading: boolean,              // Enable lazy cell loading
    virtualScrolling: boolean,         // Enable virtual scrolling
    cellBatchSize: number,             // Cells per batch
    visibilityBuffer: number           // Virtual scroll buffer
}
```

#### Core Methods

**Cell Management**
```javascript
// Add new cell
const cell = notebook.addCell(type, content, index);

// Delete cell
notebook.deleteCell(cellId);

// Run single cell
await notebook.runCell(cellId);

// Run all cells
await notebook.runAllCells();

// Clear all outputs
notebook.clearAllCells();
```

**Notebook Operations**
```javascript
// Save notebook
notebook.save();

// Load from storage
notebook.loadFromStorage();

// Export to Jupyter format
notebook.exportNotebook();

// Get notebook data
const data = notebook.getNotebookData();

// Set notebook data
notebook.setNotebookData(data);
```

**Event Handling**
```javascript
// Listen for cell execution events
notebook.on('cellExecuted', (cellId, result) => {
    console.log(`Cell ${cellId} executed:`, result);
});

// Listen for cell changes
notebook.on('cellChanged', (cellId, content) => {
    console.log(`Cell ${cellId} changed:`, content);
});

// Listen for notebook save events
notebook.on('notebookSaved', (data) => {
    console.log('Notebook saved:', data);
});
```

### WebWorker API

#### Message Types

**Execute Code**
```javascript
worker.postMessage({
    id: 'unique-message-id',
    type: 'execute',
    code: 'println("Hello, World!");',
    timeout: 30000  // Optional timeout in ms
});
```

**Reset Runtime**
```javascript
worker.postMessage({
    id: 'reset-id',
    type: 'reset'
});
```

**Get Memory Info**
```javascript
worker.postMessage({
    id: 'memory-id',
    type: 'memory_info'
});
```

#### Response Format

```javascript
// Success response
{
    id: 'message-id',
    type: 'result',
    success: true,
    result: {
        output: 'Hello, World!',
        success: true,
        execution_time_ms: 23.4,
        memory_used: 1024
    }
}

// Error response
{
    id: 'message-id',
    type: 'error',
    success: false,
    error: 'Error message'
}
```

### WASM Module API

#### WasmNotebook Class

```rust
#[wasm_bindgen]
impl WasmNotebook {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self;
    
    #[wasm_bindgen]
    pub fn execute(&mut self, code: &str) -> ExecutionResult;
    
    #[wasm_bindgen]
    pub fn reset(&mut self);
    
    #[wasm_bindgen]
    pub fn get_memory_usage(&self) -> usize;
    
    #[wasm_bindgen]
    pub fn get_runtime_ms(&self) -> f64;
}
```

#### ExecutionResult Structure

```rust
#[wasm_bindgen]
pub struct ExecutionResult {
    output: String,
    success: bool,
    execution_time_ms: f64,
    memory_used: usize,
}
```

---

## 🔧 Configuration Options

### Environment Variables

```bash
# Development
NODE_ENV=development          # Enable debug features
DEBUG_WASM=true              # Enable WASM debugging
PERFORMANCE_MONITORING=true   # Enable performance tracking

# Production
NODE_ENV=production          # Optimized builds
WASM_SIZE_LIMIT=204800       # 200KB WASM size limit
CELL_TIMEOUT=30000           # 30s cell execution timeout
```

### Build Configuration

**wasm-pack.toml**
```toml
[package]
name = "ruchy-notebook-wasm"
description = "Ruchy Notebook WASM runtime"

[build]
target = "web"

[build.profile.release]
wee_alloc = true
opt_level = "z"
debug = false

[pack]
out-dir = "pkg"

[pack.profile.release]
files = ["*.wasm", "*.js", "*.ts", "*.d.ts"]
size_limit = 204800  # 200KB limit
```

**Cargo.toml Features**
```toml
[features]
default = ["native"]
native = ["axum", "tokio", "tower", "tower-http", "arrow", "parquet"]
wasm = ["wasm-bindgen", "wasm-bindgen-futures", "web-sys", "js-sys", "bumpalo", "wee_alloc", "console_error_panic_hook"]
dataframe = ["arrow", "parquet"]
```

### Runtime Configuration

**Service Worker Config**
```javascript
// sw.js configuration
const CACHE_NAME = 'ruchy-notebook-v1.90.0';
const WASM_CACHE = 'ruchy-wasm-v1.90.0';

const STATIC_FILES = [
    './ruchy-notebook.js',
    './ruchy-worker.js',
    './styles.css',
    './pkg/ruchy_notebook.js',
    './pkg/ruchy_notebook_bg.wasm'
];
```

**PWA Manifest Config**
```json
{
    "name": "Ruchy Notebook",
    "short_name": "Ruchy",
    "description": "Interactive notebook environment for Ruchy programming language",
    "version": "1.90.0",
    "start_url": "./index.html",
    "display": "standalone",
    "theme_color": "#2D3748",
    "background_color": "#1A202C"
}
```

---

## 🔌 Plugin System

### Plugin Architecture

```javascript
class NotebookPlugin {
    constructor(notebook, options) {
        this.notebook = notebook;
        this.options = options;
    }
    
    // Called when plugin is loaded
    async onLoad() {
        // Initialize plugin
    }
    
    // Called before cell execution
    async beforeCellExecution(cellId, code) {
        // Preprocessing logic
        return code;
    }
    
    // Called after cell execution
    async afterCellExecution(cellId, result) {
        // Postprocessing logic
        return result;
    }
    
    // Called when plugin is unloaded
    async onUnload() {
        // Cleanup logic
    }
}
```

### Example: Syntax Highlighting Plugin

```javascript
class SyntaxHighlightingPlugin extends NotebookPlugin {
    async onLoad() {
        // Load syntax highlighting library
        await import('https://cdn.jsdelivr.net/npm/prismjs@1.29.0/prism.min.js');
        
        // Apply to existing cells
        this.notebook.cells.forEach(cell => {
            this.highlightCell(cell.id);
        });
        
        // Listen for new cells
        this.notebook.on('cellCreated', (cellId) => {
            this.highlightCell(cellId);
        });
    }
    
    highlightCell(cellId) {
        const cellElement = document.getElementById(cellId);
        const codeInput = cellElement.querySelector('.code-input');
        
        // Apply syntax highlighting
        Prism.highlightElement(codeInput);
    }
}

// Register plugin
notebook.registerPlugin(new SyntaxHighlightingPlugin(notebook, {
    theme: 'tomorrow-night'
}));
```

### Example: Auto-completion Plugin

```javascript
class AutoCompletePlugin extends NotebookPlugin {
    async onLoad() {
        this.completions = await this.loadCompletions();
        
        this.notebook.on('cellInput', (cellId, event) => {
            if (event.key === 'Tab') {
                this.handleTabCompletion(cellId, event);
            }
        });
    }
    
    async loadCompletions() {
        // Load Ruchy language completions
        return {
            keywords: ['let', 'fun', 'if', 'else', 'match', 'for', 'while'],
            functions: ['println', 'print', 'format', 'len', 'map', 'filter'],
            types: ['String', 'Number', 'Boolean', 'Array', 'Object']
        };
    }
    
    handleTabCompletion(cellId, event) {
        event.preventDefault();
        
        const cellElement = document.getElementById(cellId);
        const textarea = cellElement.querySelector('.code-input');
        const cursorPos = textarea.selectionStart;
        const text = textarea.value;
        
        // Find word at cursor
        const wordMatch = text.substring(0, cursorPos).match(/\w+$/);
        if (!wordMatch) return;
        
        const word = wordMatch[0];
        const suggestions = this.getSuggestions(word);
        
        if (suggestions.length > 0) {
            this.showCompletionMenu(cellId, suggestions, cursorPos);
        }
    }
    
    getSuggestions(word) {
        const all = [
            ...this.completions.keywords,
            ...this.completions.functions,
            ...this.completions.types
        ];
        
        return all.filter(item => 
            item.toLowerCase().startsWith(word.toLowerCase())
        );
    }
}
```

---

## 🧪 Testing Framework

### Unit Testing

**JavaScript Tests**
```javascript
// test/notebook.test.js
import { RuchyNotebook } from '../js/ruchy-notebook.js';

describe('RuchyNotebook', () => {
    let container, notebook;
    
    beforeEach(() => {
        container = document.createElement('div');
        document.body.appendChild(container);
        notebook = new RuchyNotebook(container);
    });
    
    afterEach(() => {
        document.body.removeChild(container);
    });
    
    test('should create notebook instance', () => {
        expect(notebook).toBeInstanceOf(RuchyNotebook);
        expect(notebook.cells).toEqual([]);
    });
    
    test('should add cell', () => {
        const cell = notebook.addCell('code', 'println("test");');
        expect(notebook.cells).toHaveLength(1);
        expect(cell.content).toBe('println("test");');
    });
    
    test('should execute cell', async () => {
        const cell = notebook.addCell('code', 'let x = 42; x');
        const result = await notebook.runCell(cell.id);
        expect(result.success).toBe(true);
        expect(result.output).toContain('42');
    });
});
```

**Rust Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    #[wasm_bindgen_test]
    fn test_wasm_notebook_creation() {
        let notebook = WasmNotebook::new();
        // Test passes if no panic
        assert!(true);
    }
    
    #[wasm_bindgen_test]
    fn test_code_execution() {
        let mut notebook = WasmNotebook::new();
        let result = notebook.execute("1 + 2");
        
        assert!(result.success());
        assert!(result.output().contains("3"));
    }
    
    #[wasm_bindgen_test]
    fn test_error_handling() {
        let mut notebook = WasmNotebook::new();
        let result = notebook.execute("invalid syntax");
        
        assert!(!result.success());
        assert!(result.output().contains("Error"));
    }
}
```

### Performance Testing

**Automated Performance Tests**
```javascript
// test/performance.test.js
describe('Performance Tests', () => {
    test('cell execution under 50ms', async () => {
        const notebook = new RuchyNotebook(container);
        const cell = notebook.addCell('code', 'let x = 42; x');
        
        const start = performance.now();
        await notebook.runCell(cell.id);
        const duration = performance.now() - start;
        
        expect(duration).toBeLessThan(50);
    });
    
    test('notebook loading under 200ms', async () => {
        const start = performance.now();
        
        // Create notebook with 100 cells
        const notebook = new RuchyNotebook(container);
        for (let i = 0; i < 100; i++) {
            notebook.addCell('code', `println("Cell ${i}");`);
        }
        
        const duration = performance.now() - start;
        expect(duration).toBeLessThan(200);
    });
});
```

### Integration Testing

**End-to-End Tests**
```javascript
// test/e2e.test.js
const { test, expect } = require('@playwright/test');

test('notebook workflow', async ({ page }) => {
    await page.goto('/');
    
    // Wait for notebook to load
    await page.waitForSelector('.ruchy-notebook');
    
    // Add cell
    await page.click('#add-cell');
    await page.fill('.code-input', 'println("Hello, World!");');
    
    // Run cell
    await page.click('.run-cell');
    
    // Check output
    await page.waitForSelector('.cell-output');
    const output = await page.textContent('.output-content');
    expect(output).toContain('Hello, World!');
});
```

---

## 🚀 Deployment Guide

### Static Hosting

**GitHub Pages**
```yaml
# .github/workflows/deploy.yml
name: Deploy to GitHub Pages

on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown
    
    - name: Install wasm-pack
      run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    
    - name: Build WASM
      run: |
        cd ruchy-notebook
        wasm-pack build --target web --out-dir pkg --release --no-default-features --features wasm
    
    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./ruchy-notebook
```

**Netlify Deployment**
```toml
# netlify.toml
[build]
  command = "cd ruchy-notebook && wasm-pack build --target web --out-dir pkg --release --no-default-features --features wasm"
  publish = "ruchy-notebook"

[build.environment]
  RUST_VERSION = "1.75"

[[headers]]
  for = "/*.wasm"
  [headers.values]
    Content-Type = "application/wasm"

[[headers]]
  for = "/sw.js"
  [headers.values]
    Cache-Control = "no-cache"
```

### Docker Deployment

**Dockerfile**
```dockerfile
# Multi-stage build for WASM
FROM rust:1.75 as wasm-builder

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /app
COPY . .

RUN cd ruchy-notebook && \
    wasm-pack build --target web --out-dir pkg --release --no-default-features --features wasm

# Nginx serving
FROM nginx:alpine

COPY --from=wasm-builder /app/ruchy-notebook /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80
```

**nginx.conf**
```nginx
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;
    
    # Enable WASM MIME type
    location ~ \.wasm$ {
        add_header Content-Type application/wasm;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Service Worker should not be cached
    location = /sw.js {
        add_header Cache-Control "no-cache";
        expires 0;
    }
    
    # Enable gzip compression
    gzip on;
    gzip_types
        application/javascript
        application/json
        application/wasm
        text/css
        text/html
        text/xml;
}
```

### CDN Integration

**jsDelivr CDN**
```html
<script type="module">
    // Load from CDN
    import { RuchyNotebook } from 'https://cdn.jsdelivr.net/npm/ruchy-notebook@1.90.0/js/ruchy-notebook.js';
    
    const container = document.getElementById('notebook');
    const notebook = new RuchyNotebook(container, {
        wasmPath: 'https://cdn.jsdelivr.net/npm/ruchy-notebook@1.90.0/pkg/ruchy_notebook_bg.wasm'
    });
</script>
```

---

## 🛠️ Advanced Integration

### React Integration

```jsx
import React, { useEffect, useRef } from 'react';
import { RuchyNotebook } from 'ruchy-notebook';

const RuchyNotebookComponent = ({ initialCells, onCellChange, ...options }) => {
    const containerRef = useRef();
    const notebookRef = useRef();
    
    useEffect(() => {
        if (containerRef.current && !notebookRef.current) {
            notebookRef.current = new RuchyNotebook(containerRef.current, options);
            
            // Load initial cells
            if (initialCells) {
                initialCells.forEach(cell => {
                    notebookRef.current.addCell(cell.type, cell.content);
                });
            }
            
            // Setup event listeners
            notebookRef.current.on('cellChanged', onCellChange);
        }
        
        return () => {
            if (notebookRef.current) {
                notebookRef.current.destroy();
            }
        };
    }, []);
    
    return <div ref={containerRef} className="ruchy-notebook-container" />;
};

export default RuchyNotebookComponent;
```

**Usage in React App**
```jsx
import RuchyNotebookComponent from './RuchyNotebookComponent';

function App() {
    const handleCellChange = (cellId, content) => {
        console.log(`Cell ${cellId} changed:`, content);
    };
    
    const initialCells = [
        { type: 'code', content: 'println("Hello from React!");' }
    ];
    
    return (
        <div className="App">
            <h1>My Ruchy Notebook App</h1>
            <RuchyNotebookComponent 
                initialCells={initialCells}
                onCellChange={handleCellChange}
                theme="dark"
                autoSave={true}
            />
        </div>
    );
}
```

### Vue Integration

```vue
<template>
    <div ref="notebookContainer" class="ruchy-notebook-container"></div>
</template>

<script>
import { RuchyNotebook } from 'ruchy-notebook';

export default {
    name: 'RuchyNotebook',
    props: {
        initialCells: Array,
        options: Object
    },
    data() {
        return {
            notebook: null
        };
    },
    mounted() {
        this.notebook = new RuchyNotebook(this.$refs.notebookContainer, {
            ...this.options,
            onCellChange: this.handleCellChange
        });
        
        if (this.initialCells) {
            this.initialCells.forEach(cell => {
                this.notebook.addCell(cell.type, cell.content);
            });
        }
    },
    beforeUnmount() {
        if (this.notebook) {
            this.notebook.destroy();
        }
    },
    methods: {
        handleCellChange(cellId, content) {
            this.$emit('cell-changed', { cellId, content });
        },
        
        addCell(type, content) {
            return this.notebook.addCell(type, content);
        },
        
        runCell(cellId) {
            return this.notebook.runCell(cellId);
        },
        
        exportNotebook() {
            return this.notebook.exportNotebook();
        }
    }
};
</script>
```

### Node.js Integration

```javascript
// server.js - Node.js server with Ruchy Notebook
const express = require('express');
const path = require('path');
const fs = require('fs');

const app = express();
const PORT = 3000;

// Serve static files
app.use('/notebook', express.static(path.join(__dirname, 'ruchy-notebook')));

// API endpoints
app.use(express.json());

// Save notebook endpoint
app.post('/api/notebooks/:id', (req, res) => {
    const { id } = req.params;
    const notebook = req.body;
    
    try {
        fs.writeFileSync(`./notebooks/${id}.json`, JSON.stringify(notebook, null, 2));
        res.json({ success: true, message: 'Notebook saved' });
    } catch (error) {
        res.status(500).json({ success: false, error: error.message });
    }
});

// Load notebook endpoint
app.get('/api/notebooks/:id', (req, res) => {
    const { id } = req.params;
    
    try {
        const data = fs.readFileSync(`./notebooks/${id}.json`, 'utf8');
        res.json(JSON.parse(data));
    } catch (error) {
        res.status(404).json({ success: false, error: 'Notebook not found' });
    }
});

// List notebooks endpoint
app.get('/api/notebooks', (req, res) => {
    try {
        const files = fs.readdirSync('./notebooks')
            .filter(file => file.endsWith('.json'))
            .map(file => ({
                id: file.replace('.json', ''),
                name: file.replace('.json', ''),
                modified: fs.statSync(`./notebooks/${file}`).mtime
            }));
        
        res.json(files);
    } catch (error) {
        res.status(500).json({ success: false, error: error.message });
    }
});

app.listen(PORT, () => {
    console.log(`Ruchy Notebook server running on http://localhost:${PORT}`);
});
```

---

## 📊 Performance Optimization

### WASM Optimization

**Build Optimization**
```toml
[profile.release]
opt-level = "z"           # Optimize for size
lto = true                # Link-time optimization
codegen-units = 1         # Single codegen unit
strip = true              # Strip debug symbols

[profile.wasm]
inherits = "release"
opt-level = "z"
panic = "abort"           # Smaller panic handler
```

**Memory Optimization**
```rust
// Use wee_alloc for smaller WASM size
use wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Optimize memory usage patterns
impl WasmNotebook {
    pub fn execute(&mut self, code: &str) -> ExecutionResult {
        // Reuse allocations where possible
        self.scratch_buffer.clear();
        
        // Use stack allocation for small data
        let mut compiler = Compiler::new();
        
        // Explicit memory management
        let result = match compiler.compile_expression(code) {
            Ok(module) => self.vm.execute(&module),
            Err(e) => Err(e)
        };
        
        // Clear temporary data
        self.vm.clear_stack();
        
        result.into()
    }
}
```

### JavaScript Optimization

**Lazy Loading Implementation**
```javascript
class RuchyNotebook {
    constructor(container, options) {
        this.options = {
            lazyLoading: true,
            virtualScrolling: true,
            cellBatchSize: 10,
            ...options
        };
        
        this.intersectionObserver = new IntersectionObserver(
            this.handleIntersection.bind(this),
            { rootMargin: '100px' }
        );
    }
    
    renderCell(cell, lazy = false) {
        const cellElement = document.createElement('div');
        cellElement.id = cell.id;
        cellElement.dataset.cellId = cell.id;
        
        if (lazy && this.options.lazyLoading) {
            // Render placeholder
            cellElement.innerHTML = `<div class="cell-placeholder">Loading...</div>`;
            this.intersectionObserver.observe(cellElement);
        } else {
            // Render full cell
            this.renderFullCell(cellElement, cell);
        }
        
        return cellElement;
    }
    
    handleIntersection(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const cellId = entry.target.dataset.cellId;
                const cell = this.cells.find(c => c.id === cellId);
                
                if (cell && entry.target.classList.contains('cell-placeholder')) {
                    this.renderFullCell(entry.target, cell);
                    this.intersectionObserver.unobserve(entry.target);
                }
            }
        });
    }
}
```

**WebWorker Pool**
```javascript
class WorkerPool {
    constructor(workerScript, poolSize = 4) {
        this.workers = [];
        this.available = [];
        this.queue = [];
        
        for (let i = 0; i < poolSize; i++) {
            const worker = new Worker(workerScript);
            worker.onmessage = this.handleWorkerMessage.bind(this);
            this.workers.push(worker);
            this.available.push(worker);
        }
    }
    
    async execute(code, timeout = 30000) {
        return new Promise((resolve, reject) => {
            const task = { code, timeout, resolve, reject };
            
            if (this.available.length > 0) {
                this.runTask(task);
            } else {
                this.queue.push(task);
            }
        });
    }
    
    runTask(task) {
        const worker = this.available.pop();
        const messageId = Date.now().toString();
        
        task.worker = worker;
        task.messageId = messageId;
        
        const timeoutId = setTimeout(() => {
            task.reject(new Error('Execution timeout'));
            this.releaseWorker(worker);
        }, task.timeout);
        
        task.timeoutId = timeoutId;
        
        worker.postMessage({
            id: messageId,
            type: 'execute',
            code: task.code
        });
    }
    
    handleWorkerMessage(event) {
        const worker = event.target;
        const task = this.findTaskByWorker(worker);
        
        if (task) {
            clearTimeout(task.timeoutId);
            
            if (event.data.success) {
                task.resolve(event.data.result);
            } else {
                task.reject(new Error(event.data.error));
            }
            
            this.releaseWorker(worker);
        }
    }
    
    releaseWorker(worker) {
        this.available.push(worker);
        
        if (this.queue.length > 0) {
            const nextTask = this.queue.shift();
            this.runTask(nextTask);
        }
    }
}
```

---

## 🐛 Debugging Guide

### Browser Developer Tools

**Console Debugging**
```javascript
// Enable debug mode
window.RUCHY_DEBUG = true;

// Debug cell execution
notebook.on('cellExecuted', (cellId, result) => {
    if (window.RUCHY_DEBUG) {
        console.group(`Cell ${cellId} Execution`);
        console.log('Result:', result);
        console.log('Execution Time:', result.execution_time_ms + 'ms');
        console.log('Memory Used:', result.memory_used + ' bytes');
        console.groupEnd();
    }
});

// Debug WASM loading
console.time('WASM Load');
await notebook.loadWasm();
console.timeEnd('WASM Load');
```

**Performance Profiling**
```javascript
// Profile cell execution
async function profileCellExecution(notebook, cellId) {
    const cell = notebook.cells.find(c => c.id === cellId);
    
    console.time(`Cell ${cellId} Total`);
    
    // Mark compilation start
    performance.mark('compile-start');
    
    const result = await notebook.runCell(cellId);
    
    // Mark compilation end
    performance.mark('compile-end');
    performance.measure('compilation', 'compile-start', 'compile-end');
    
    console.timeEnd(`Cell ${cellId} Total`);
    
    // Get all performance entries
    const entries = performance.getEntriesByType('measure');
    entries.forEach(entry => {
        console.log(`${entry.name}: ${entry.duration.toFixed(2)}ms`);
    });
    
    return result;
}
```

### WASM Debugging

**Rust Debug Builds**
```toml
[profile.dev]
debug = true
opt-level = 0

[profile.release]
debug = true  # Keep debug info for profiling
opt-level = "z"
```

**Console Logging from Rust**
```rust
// Enable panic hooks for better error messages
#[wasm_bindgen]
pub fn init_panic_hook() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// Logging macro for WASM
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

impl WasmNotebook {
    pub fn execute(&mut self, code: &str) -> ExecutionResult {
        console_log!("Executing code: {}", code);
        
        let start = performance_now();
        let result = self.vm.execute_internal(code);
        let duration = performance_now() - start;
        
        console_log!("Execution completed in {}ms", duration);
        
        result
    }
}
```

### Memory Debugging

**Memory Usage Tracking**
```javascript
class MemoryTracker {
    constructor(notebook) {
        this.notebook = notebook;
        this.snapshots = [];
        this.tracking = false;
    }
    
    startTracking() {
        this.tracking = true;
        this.trackingInterval = setInterval(() => {
            if (performance.memory) {
                this.snapshots.push({
                    timestamp: Date.now(),
                    used: performance.memory.usedJSHeapSize,
                    total: performance.memory.totalJSHeapSize,
                    limit: performance.memory.jsHeapSizeLimit
                });
            }
        }, 1000);
    }
    
    stopTracking() {
        this.tracking = false;
        if (this.trackingInterval) {
            clearInterval(this.trackingInterval);
        }
    }
    
    getMemoryReport() {
        if (this.snapshots.length < 2) {
            return 'Insufficient data';
        }
        
        const first = this.snapshots[0];
        const last = this.snapshots[this.snapshots.length - 1];
        const increase = last.used - first.used;
        const duration = last.timestamp - first.timestamp;
        
        return {
            memoryIncrease: increase / 1024 / 1024, // MB
            duration: duration / 1000, // seconds
            rate: (increase / duration) * 1000, // bytes per second
            snapshots: this.snapshots
        };
    }
    
    detectLeaks() {
        const report = this.getMemoryReport();
        const threshold = 10 * 1024 * 1024; // 10MB
        
        if (report.memoryIncrease > threshold) {
            console.warn(`Potential memory leak detected: ${report.memoryIncrease.toFixed(2)}MB increase`);
            return true;
        }
        
        return false;
    }
}

// Usage
const tracker = new MemoryTracker(notebook);
tracker.startTracking();

// Run some operations...
setTimeout(() => {
    tracker.stopTracking();
    console.log('Memory Report:', tracker.getMemoryReport());
    
    if (tracker.detectLeaks()) {
        console.error('Memory leak detected!');
    }
}, 30000);
```

### Error Reporting

**Structured Error Handling**
```javascript
class ErrorReporter {
    constructor(notebook) {
        this.notebook = notebook;
        this.errors = [];
        this.setupErrorHandling();
    }
    
    setupErrorHandling() {
        // Catch unhandled errors
        window.addEventListener('error', (event) => {
            this.reportError({
                type: 'javascript',
                message: event.message,
                filename: event.filename,
                lineno: event.lineno,
                colno: event.colno,
                stack: event.error?.stack,
                timestamp: new Date().toISOString()
            });
        });
        
        // Catch unhandled promise rejections
        window.addEventListener('unhandledrejection', (event) => {
            this.reportError({
                type: 'promise',
                message: event.reason?.message || event.reason,
                stack: event.reason?.stack,
                timestamp: new Date().toISOString()
            });
        });
        
        // Catch WASM execution errors
        this.notebook.on('executionError', (error) => {
            this.reportError({
                type: 'wasm',
                message: error.message,
                code: error.code,
                cellId: error.cellId,
                timestamp: new Date().toISOString()
            });
        });
    }
    
    reportError(error) {
        this.errors.push(error);
        
        // Log to console in development
        if (window.RUCHY_DEBUG) {
            console.error('Ruchy Notebook Error:', error);
        }
        
        // Send to error tracking service in production
        if (this.errorTrackingEnabled) {
            this.sendToErrorService(error);
        }
    }
    
    getErrorReport() {
        return {
            errors: this.errors,
            browser: navigator.userAgent,
            platform: navigator.platform,
            timestamp: new Date().toISOString(),
            notebookVersion: this.notebook.version
        };
    }
    
    clearErrors() {
        this.errors = [];
    }
}
```

---

*This developer guide covers Ruchy Notebook v1.90.0 architecture and integration patterns. For the latest API changes and features, check the GitHub repository.*