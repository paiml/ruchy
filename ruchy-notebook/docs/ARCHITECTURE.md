# 🏗️ Ruchy Notebook Architecture Documentation

**Version 1.90.0** - Detailed system architecture and design decisions

---

## 📋 Table of Contents

- [🎯 Design Philosophy](#-design-philosophy)
- [🏗️ System Architecture](#️-system-architecture)
- [🔧 Component Design](#-component-design)
- [💾 Data Flow](#-data-flow)
- [🚀 Performance Architecture](#-performance-architecture)
- [🔒 Security Model](#-security-model)
- [📱 Progressive Web App](#-progressive-web-app)
- [🔮 Future Architecture](#-future-architecture)

---

## 🎯 Design Philosophy

### Core Principles

**1. Performance First**
- **Target**: <50ms cell execution, <200KB WASM bundle
- **Strategy**: WebAssembly + WebWorkers + aggressive optimization
- **Measurement**: Continuous performance monitoring and regression detection

**2. Progressive Enhancement**
- **Base**: Works with JavaScript disabled (static content)
- **Enhanced**: Full interactive experience with modern browsers
- **Graceful**: Degrades gracefully on unsupported platforms

**3. Offline-First**
- **Service Worker**: Caches all essential assets
- **Local Storage**: Persistent notebook state
- **PWA**: Installable app experience

**4. Modular Architecture**
- **Separation**: Clear boundaries between UI, execution, and data layers
- **Extensibility**: Plugin system for custom functionality
- **Testing**: Each component independently testable

**5. Web Standards Compliance**
- **Modern APIs**: WebAssembly, WebWorkers, Service Workers
- **Accessibility**: WCAG 2.1 AA compliance
- **Security**: Content Security Policy, secure contexts

---

## 🏗️ System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser Environment                      │
├─────────────────────────────────────────────────────────────┤
│  Main Thread          │  WebWorker Thread  │  Service Worker │
│                       │                    │                 │
│  ┌─────────────────┐  │  ┌───────────────┐ │  ┌─────────────┐│
│  │   RuchyNotebook │  │  │ ExecutionHost │ │  │ CacheManager││
│  │   (UI Layer)    │  │  │ (WASM Runtime)│ │  │ (Offline)   ││
│  └─────────────────┘  │  └───────────────┘ │  └─────────────┘│
│           │            │         │          │        │        │
│  ┌─────────────────┐  │  ┌───────────────┐ │  ┌─────────────┐│
│  │  DOM Renderer   │  │  │ VirtualMachine│ │  │  Asset      ││
│  │  (Cells/Output) │  │  │ (Interpreter) │ │  │  Storage    ││
│  └─────────────────┘  │  └───────────────┘ │  └─────────────┘│
│           │            │         │          │        │        │
│  ┌─────────────────┐  │  ┌───────────────┐ │  ┌─────────────┐│
│  │  Event System   │  │  │ Memory Mgmt   │ │  │ Version     ││
│  │  (Interactions) │  │  │ (Arena/Slab)  │ │  │ Control     ││
│  └─────────────────┘  │  └───────────────┘ │  └─────────────┘│
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                    Data Layer                               │
├─────────────────────────────────────────────────────────────┤
│  Local Storage     │  IndexedDB         │  External APIs     │
│  ┌─────────────┐   │  ┌─────────────┐   │  ┌─────────────┐   │
│  │ Notebooks   │   │  │ Large Files │   │  │ Remote Data │   │
│  │ Preferences │   │  │ Media Assets│   │  │ Sharing     │   │
│  │ Session     │   │  │ Cache Data  │   │  │ Cloud Sync  │   │
│  └─────────────┘   │  └─────────────┘   │  └─────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Thread Architecture

**Main Thread (UI)**
- **Responsibility**: User interface, DOM manipulation, event handling
- **Performance**: Optimized for 60fps rendering
- **Communication**: Message passing with WebWorker

**WebWorker Thread (Execution)**
- **Responsibility**: Code compilation and execution
- **Isolation**: Prevents UI blocking during computation
- **WASM Runtime**: Ruchy language interpreter

**Service Worker (Caching)**
- **Responsibility**: Asset caching, offline functionality
- **Lifecycle**: Independent of main application
- **Updates**: Handles version management and cache invalidation

---

## 🔧 Component Design

### Frontend Layer (JavaScript)

#### RuchyNotebook (Main Controller)

```javascript
class RuchyNotebook {
    constructor(container, options) {
        // Core components
        this.container = container;
        this.cells = [];
        this.eventSystem = new EventSystem();
        this.renderer = new CellRenderer(container);
        this.storage = new StorageManager();
        this.workerPool = new WorkerPool();
        
        // Performance optimizations
        this.virtualScroller = new VirtualScroller();
        this.intersectionObserver = new LazyLoader();
        
        // Initialize subsystems
        this.init();
    }
    
    // Public API methods
    addCell(type, content, index) { /* ... */ }
    runCell(cellId) { /* ... */ }
    exportNotebook() { /* ... */ }
    
    // Private implementation
    private init() { /* ... */ }
    private setupEventHandlers() { /* ... */ }
    private scheduleAutoSave() { /* ... */ }
}
```

**Design Decisions:**
- **Single Responsibility**: Each class handles one aspect
- **Dependency Injection**: Components passed in constructor
- **Event-Driven**: Loose coupling via event system
- **Performance**: Virtual scrolling and lazy loading built-in

#### Cell Renderer (DOM Management)

```javascript
class CellRenderer {
    constructor(container) {
        this.container = container;
        this.cellElements = new Map();
        this.templateCache = new Map();
    }
    
    renderCell(cell, options = {}) {
        const { lazy = false, virtual = false } = options;
        
        if (lazy && !this.isVisible(cell)) {
            return this.renderPlaceholder(cell);
        }
        
        const template = this.getTemplate(cell.type);
        const element = template.cloneNode(true);
        
        this.populateCell(element, cell);
        this.attachEventHandlers(element, cell);
        
        if (virtual) {
            this.setupVirtualization(element, cell);
        }
        
        return element;
    }
    
    private getTemplate(cellType) {
        if (!this.templateCache.has(cellType)) {
            this.templateCache.set(cellType, this.createTemplate(cellType));
        }
        return this.templateCache.get(cellType);
    }
}
```

**Design Decisions:**
- **Template Caching**: Avoid DOM creation overhead
- **Virtual DOM**: Only render visible cells
- **Event Delegation**: Efficient event handling
- **Performance**: Minimize layout thrashing

### Backend Layer (Rust/WASM)

#### Virtual Machine (Core Interpreter)

```rust
pub struct VirtualMachine {
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    memory: ArenaAllocator,
    gc: GarbageCollector,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(64),
            globals: HashMap::new(),
            memory: ArenaAllocator::new(256_000), // 256KB
            gc: GarbageCollector::new(),
        }
    }
    
    pub fn execute(&mut self, bytecode: &BytecodeModule) -> Result<ExecutionResult, RuntimeError> {
        let start_time = performance_now();
        
        // Execute bytecode instructions
        let result = self.run_instructions(&bytecode.instructions)?;
        
        // Collect garbage if needed
        if self.memory.should_collect() {
            self.gc.collect(&mut self.memory);
        }
        
        let execution_time = performance_now() - start_time;
        
        Ok(ExecutionResult {
            value: result,
            output: self.get_output(),
            execution_time_ms: execution_time,
            memory_used: self.memory.bytes_used(),
        })
    }
    
    fn run_instructions(&mut self, instructions: &[OpCode]) -> Result<Value, RuntimeError> {
        let mut pc = 0; // Program counter
        
        while pc < instructions.len() {
            match instructions[pc] {
                OpCode::Push(value) => self.stack.push(value),
                OpCode::Add => {
                    let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                    self.stack.push(a + b);
                }
                OpCode::Call(func_id) => {
                    self.call_function(func_id)?;
                }
                // ... other opcodes
            }
            pc += 1;
        }
        
        self.stack.pop().unwrap_or(Value::Nil)
    }
}
```

**Design Decisions:**
- **Stack-Based**: Efficient bytecode execution
- **Arena Allocation**: Fast memory management
- **Garbage Collection**: Automatic memory cleanup
- **Performance Monitoring**: Built-in timing and memory tracking

#### Memory Management

```rust
pub struct ArenaAllocator {
    storage: RefCell<Vec<Box<dyn Any>>>,
    bytes_used: Cell<usize>,
    capacity: usize,
    generation: Cell<u64>,
}

impl ArenaAllocator {
    pub fn new(capacity: usize) -> Self {
        Self {
            storage: RefCell::new(Vec::new()),
            bytes_used: Cell::new(0),
            capacity,
            generation: Cell::new(0),
        }
    }
    
    pub fn alloc<T: 'static>(&self, value: T) -> Handle<T> {
        let size = std::mem::size_of::<T>();
        
        if self.bytes_used.get() + size > self.capacity {
            self.trigger_gc();
        }
        
        let boxed = Box::new(value);
        let index = self.storage.borrow().len();
        
        self.storage.borrow_mut().push(boxed);
        self.bytes_used.set(self.bytes_used.get() + size);
        
        Handle::new(index, self.generation.get())
    }
    
    pub fn should_collect(&self) -> bool {
        self.bytes_used.get() > self.capacity / 2
    }
    
    fn trigger_gc(&self) {
        // Increment generation to invalidate old handles
        self.generation.set(self.generation.get() + 1);
        
        // Clear storage
        self.storage.borrow_mut().clear();
        self.bytes_used.set(0);
    }
}

pub struct Handle<T> {
    index: usize,
    generation: u64,
    phantom: std::marker::PhantomData<T>,
}
```

**Design Decisions:**
- **Safe Rust**: No unsafe code, use Rc<RefCell<T>> patterns
- **Generation-Based**: Handles become invalid after GC
- **Arena Pattern**: Fast allocation, bulk deallocation
- **Size Tracking**: Monitor memory usage for GC triggers

### Communication Layer

#### Message Protocol (WebWorker)

```typescript
// Message types sent to WebWorker
interface WorkerRequest {
    id: string;
    type: 'execute' | 'reset' | 'memory_info';
    payload?: {
        code?: string;
        timeout?: number;
        context?: ExecutionContext;
    };
}

// Message types received from WebWorker
interface WorkerResponse {
    id: string;
    type: 'result' | 'error' | 'progress';
    success: boolean;
    data?: {
        output?: string;
        value?: any;
        execution_time_ms?: number;
        memory_used?: number;
        error?: string;
    };
}
```

**Implementation:**
```javascript
class WorkerCommunicator {
    constructor(workerScript) {
        this.worker = new Worker(workerScript);
        this.pendingRequests = new Map();
        this.messageId = 0;
        
        this.worker.onmessage = this.handleResponse.bind(this);
        this.worker.onerror = this.handleError.bind(this);
    }
    
    async sendRequest(type, payload, timeout = 30000) {
        const id = (++this.messageId).toString();
        
        return new Promise((resolve, reject) => {
            // Store request for response handling
            this.pendingRequests.set(id, { resolve, reject });
            
            // Set timeout
            setTimeout(() => {
                if (this.pendingRequests.has(id)) {
                    this.pendingRequests.delete(id);
                    reject(new Error('Request timeout'));
                }
            }, timeout);
            
            // Send message
            this.worker.postMessage({ id, type, payload });
        });
    }
    
    handleResponse(event) {
        const { id, success, data, error } = event.data;
        const request = this.pendingRequests.get(id);
        
        if (request) {
            this.pendingRequests.delete(id);
            
            if (success) {
                request.resolve(data);
            } else {
                request.reject(new Error(error));
            }
        }
    }
}
```

**Design Decisions:**
- **Type Safety**: TypeScript interfaces for messages
- **Request/Response**: Async/await pattern for easier usage
- **Timeout Handling**: Prevent hanging requests
- **Error Recovery**: Graceful handling of worker failures

---

## 💾 Data Flow

### Execution Flow

```
User Input → Cell → UI Update → Worker Request → WASM Execution → Response → UI Update
     │         │         │            │              │           │         │
     │         │         │            │              │           │         └─ DOM update
     │         │         │            │              │           └─ Parse result
     │         │         │            │              └─ Run bytecode
     │         │         │            └─ Compile to bytecode
     │         │         └─ Show "Running..." state
     │         └─ Validate and format code
     └─ Keyboard/mouse events
```

### Storage Architecture

```
┌─────────────────────────────────────────────┐
│              Storage Hierarchy              │
├─────────────────────────────────────────────┤
│  Session (Memory)                           │
│  ├─ Current notebook state                  │
│  ├─ Undo/redo history                       │
│  ├─ Active cell selections                  │
│  └─ Temporary variables                     │
├─────────────────────────────────────────────┤
│  Local Storage (5-10MB)                     │
│  ├─ Auto-saved notebooks                    │
│  ├─ User preferences                        │
│  ├─ Recent notebooks list                   │
│  └─ Performance metrics                     │
├─────────────────────────────────────────────┤
│  IndexedDB (50MB-2GB)                       │
│  ├─ Large notebook files                    │
│  ├─ Media attachments                       │
│  ├─ Cached computation results              │
│  └─ Offline data sets                       │
├─────────────────────────────────────────────┤
│  Service Worker Cache                       │
│  ├─ Application assets                      │
│  ├─ WASM modules                            │
│  ├─ Static resources                        │
│  └─ API responses                           │
└─────────────────────────────────────────────┘
```

### State Management

```javascript
class NotebookState {
    constructor() {
        this.state = {
            notebook: {
                cells: [],
                metadata: {},
                kernelState: {}
            },
            ui: {
                selectedCell: null,
                theme: 'dark',
                layout: 'default'
            },
            execution: {
                isRunning: false,
                queue: [],
                results: new Map()
            }
        };
        
        this.subscribers = new Set();
        this.history = [];
        this.historyIndex = 0;
    }
    
    // Immutable state updates
    updateState(path, value) {
        const newState = this.deepClone(this.state);
        this.setNestedValue(newState, path, value);
        
        // Save to history
        this.history.push(this.state);
        if (this.history.length > 100) {
            this.history.shift();
        }
        
        this.state = newState;
        this.notifySubscribers();
        this.persistState();
    }
    
    // Subscribe to state changes
    subscribe(callback) {
        this.subscribers.add(callback);
        return () => this.subscribers.delete(callback);
    }
    
    // Undo/redo functionality
    undo() {
        if (this.historyIndex > 0) {
            this.historyIndex--;
            this.state = this.history[this.historyIndex];
            this.notifySubscribers();
        }
    }
    
    redo() {
        if (this.historyIndex < this.history.length - 1) {
            this.historyIndex++;
            this.state = this.history[this.historyIndex];
            this.notifySubscribers();
        }
    }
}
```

**Design Decisions:**
- **Immutable Updates**: Prevent accidental mutations
- **History Management**: Built-in undo/redo support
- **Reactive**: Automatic UI updates on state changes
- **Persistence**: Automatic saving to local storage

---

## 🚀 Performance Architecture

### Optimization Strategies

#### 1. WebAssembly Optimization

**Build Pipeline:**
```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Single compilation unit
panic = "abort"        # Smaller panic handler
strip = true           # Remove debug symbols

[dependencies]
wee_alloc = "0.4"      # Smaller allocator
```

**Runtime Optimization:**
```rust
// Use const generics for compile-time optimization
impl<const N: usize> Stack<N> {
    pub fn push(&mut self, value: Value) {
        if self.len < N {
            self.data[self.len] = value;
            self.len += 1;
        }
    }
}

// Inline hot paths
#[inline(always)]
pub fn add_values(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
        _ => Value::Error("Type mismatch".to_string())
    }
}
```

#### 2. Virtual Scrolling Implementation

```javascript
class VirtualScroller {
    constructor(container, itemHeight = 150) {
        this.container = container;
        this.itemHeight = itemHeight;
        this.visibleRange = { start: 0, end: 0 };
        this.scrollTop = 0;
        
        this.setupScrollListener();
    }
    
    updateVisibleRange(items) {
        const containerHeight = this.container.clientHeight;
        const startIndex = Math.floor(this.scrollTop / this.itemHeight);
        const endIndex = Math.min(
            items.length,
            startIndex + Math.ceil(containerHeight / this.itemHeight) + 5 // Buffer
        );
        
        this.visibleRange = { start: startIndex, end: endIndex };
        return this.visibleRange;
    }
    
    renderVisibleItems(items, renderFn) {
        const { start, end } = this.visibleRange;
        const visibleItems = items.slice(start, end);
        
        // Create spacers for non-visible items
        const topSpacer = this.createSpacer(start * this.itemHeight);
        const bottomSpacer = this.createSpacer((items.length - end) * this.itemHeight);
        
        // Render visible items
        const fragment = document.createDocumentFragment();
        fragment.appendChild(topSpacer);
        
        visibleItems.forEach((item, index) => {
            const element = renderFn(item, start + index);
            fragment.appendChild(element);
        });
        
        fragment.appendChild(bottomSpacer);
        
        // Replace container contents
        this.container.innerHTML = '';
        this.container.appendChild(fragment);
    }
}
```

#### 3. Memory Pool Pattern

```rust
pub struct ObjectPool<T> {
    objects: Vec<T>,
    available: Vec<usize>,
    factory: Box<dyn Fn() -> T>,
}

impl<T> ObjectPool<T> {
    pub fn new<F>(size: usize, factory: F) -> Self 
    where 
        F: Fn() -> T + 'static 
    {
        let objects: Vec<T> = (0..size).map(|_| factory()).collect();
        let available: Vec<usize> = (0..size).collect();
        
        Self {
            objects,
            available,
            factory: Box::new(factory),
        }
    }
    
    pub fn acquire(&mut self) -> Option<PooledObject<T>> {
        if let Some(index) = self.available.pop() {
            Some(PooledObject::new(index, self))
        } else {
            None
        }
    }
    
    pub fn release(&mut self, index: usize) {
        self.available.push(index);
    }
}

pub struct PooledObject<'a, T> {
    index: usize,
    pool: &'a mut ObjectPool<T>,
}

impl<T> Drop for PooledObject<'_, T> {
    fn drop(&mut self) {
        self.pool.release(self.index);
    }
}
```

### Performance Monitoring

```javascript
class PerformanceMonitor {
    constructor() {
        this.metrics = {
            cellExecutions: [],
            memoryUsage: [],
            renderTimes: [],
            userInteractions: []
        };
        
        this.setupObservers();
    }
    
    recordCellExecution(duration, memoryUsed) {
        this.metrics.cellExecutions.push({
            timestamp: Date.now(),
            duration,
            memoryUsed
        });
        
        // Alert if performance degrades
        if (duration > 50) {
            console.warn(`Slow cell execution: ${duration}ms`);
            this.analyzePerformanceIssues();
        }
    }
    
    setupObservers() {
        // Memory usage monitoring
        if (performance.memory) {
            setInterval(() => {
                this.metrics.memoryUsage.push({
                    timestamp: Date.now(),
                    used: performance.memory.usedJSHeapSize,
                    total: performance.memory.totalJSHeapSize
                });
            }, 5000);
        }
        
        // Long task detection
        if ('PerformanceObserver' in window) {
            const observer = new PerformanceObserver((list) => {
                list.getEntries().forEach((entry) => {
                    if (entry.duration > 50) {
                        console.warn(`Long task detected: ${entry.duration}ms`);
                    }
                });
            });
            observer.observe({ entryTypes: ['longtask'] });
        }
    }
    
    generateReport() {
        const cellExecStats = this.calculateStats(
            this.metrics.cellExecutions.map(e => e.duration)
        );
        
        return {
            cellExecution: {
                average: cellExecStats.mean,
                p95: cellExecStats.p95,
                target: 50,
                passing: cellExecStats.mean < 50
            },
            memoryUsage: {
                current: performance.memory?.usedJSHeapSize || 0,
                peak: Math.max(...this.metrics.memoryUsage.map(m => m.used))
            },
            recommendations: this.generateRecommendations()
        };
    }
}
```

---

## 🔒 Security Model

### Content Security Policy

```html
<meta http-equiv="Content-Security-Policy" content="
    default-src 'self';
    script-src 'self' 'wasm-unsafe-eval';
    worker-src 'self';
    style-src 'self' 'unsafe-inline';
    img-src 'self' data: blob:;
    connect-src 'self';
    font-src 'self';
">
```

### WASM Sandbox

```rust
// Restrict system access in WASM environment
#[cfg(target_arch = "wasm32")]
mod wasm_security {
    use super::*;
    
    // No file system access
    pub fn read_file(_path: &str) -> Result<String, SecurityError> {
        Err(SecurityError::AccessDenied("File access not permitted"))
    }
    
    // No network access
    pub fn fetch_url(_url: &str) -> Result<String, SecurityError> {
        Err(SecurityError::AccessDenied("Network access not permitted"))
    }
    
    // Limited execution time
    pub fn execute_with_timeout<F, R>(f: F, timeout_ms: u32) -> Result<R, SecurityError>
    where 
        F: FnOnce() -> R 
    {
        // Implementation would use web APIs for timeout
        todo!("Implement WASM timeout mechanism")
    }
}
```

### Input Validation

```rust
pub struct CodeValidator {
    max_recursion_depth: usize,
    max_loop_iterations: usize,
    forbidden_imports: Vec<String>,
}

impl CodeValidator {
    pub fn validate(&self, code: &str) -> Result<(), ValidationError> {
        let ast = self.parse_code(code)?;
        
        self.check_recursion_depth(&ast)?;
        self.check_loop_bounds(&ast)?;
        self.check_forbidden_operations(&ast)?;
        
        Ok(())
    }
    
    fn check_recursion_depth(&self, ast: &AST) -> Result<(), ValidationError> {
        let mut depth = 0;
        self.visit_recursive_calls(ast, &mut depth)?;
        
        if depth > self.max_recursion_depth {
            return Err(ValidationError::ExcessiveRecursion(depth));
        }
        
        Ok(())
    }
    
    fn check_forbidden_operations(&self, ast: &AST) -> Result<(), ValidationError> {
        // Check for potentially unsafe operations
        for node in ast.nodes() {
            match node {
                Node::Import(module) if self.forbidden_imports.contains(module) => {
                    return Err(ValidationError::ForbiddenImport(module.clone()));
                }
                Node::SystemCall(_) => {
                    return Err(ValidationError::ForbiddenOperation("System calls not allowed"));
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

---

## 📱 Progressive Web App

### Manifest Configuration

```json
{
    "name": "Ruchy Notebook",
    "short_name": "Ruchy",
    "description": "Interactive notebook environment for Ruchy programming language",
    "version": "1.90.0",
    "start_url": "./index.html",
    "display": "standalone",
    "orientation": "any",
    "theme_color": "#2D3748",
    "background_color": "#1A202C",
    "scope": "./",
    
    "icons": [
        {
            "src": "./icons/icon-192x192.png",
            "sizes": "192x192",
            "type": "image/png",
            "purpose": "maskable any"
        },
        {
            "src": "./icons/icon-512x512.png",
            "sizes": "512x512",
            "type": "image/png",
            "purpose": "maskable any"
        }
    ],
    
    "file_handlers": [
        {
            "action": "./index.html",
            "accept": {
                "application/x-ipynb+json": [".ipynb"],
                "text/x-ruchy": [".ruchy", ".rcy"]
            }
        }
    ],
    
    "share_target": {
        "action": "./share.html",
        "method": "POST",
        "enctype": "multipart/form-data",
        "params": {
            "files": [
                {
                    "name": "notebook",
                    "accept": ["application/x-ipynb+json", "text/x-ruchy"]
                }
            ]
        }
    }
}
```

### Service Worker Architecture

```javascript
// sw.js - Service Worker
const CACHE_VERSION = 'v1.90.0';
const STATIC_CACHE = `ruchy-notebook-static-${CACHE_VERSION}`;
const DYNAMIC_CACHE = `ruchy-notebook-dynamic-${CACHE_VERSION}`;

// Caching strategies
const CACHE_STRATEGIES = {
    static: 'cache-first',
    api: 'network-first',
    images: 'cache-first',
    wasm: 'cache-first'
};

self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(STATIC_CACHE)
            .then((cache) => {
                return cache.addAll([
                    './',
                    './index.html',
                    './js/ruchy-notebook.js',
                    './js/ruchy-worker.js',
                    './pkg/ruchy_notebook.js',
                    './pkg/ruchy_notebook_bg.wasm',
                    './styles.css',
                    './manifest.json'
                ]);
            })
    );
});

self.addEventListener('fetch', (event) => {
    const { request } = event;
    const url = new URL(request.url);
    
    // Route requests based on type
    if (url.pathname.endsWith('.wasm')) {
        event.respondWith(handleWasmRequest(request));
    } else if (url.pathname.startsWith('/api/')) {
        event.respondWith(handleApiRequest(request));
    } else {
        event.respondWith(handleStaticRequest(request));
    }
});

async function handleWasmRequest(request) {
    const cache = await caches.open(STATIC_CACHE);
    const cachedResponse = await cache.match(request);
    
    if (cachedResponse) {
        return cachedResponse;
    }
    
    try {
        const response = await fetch(request);
        if (response.ok) {
            cache.put(request, response.clone());
        }
        return response;
    } catch (error) {
        console.error('WASM fetch failed:', error);
        throw error;
    }
}
```

### Installation Prompt

```javascript
class PWAInstaller {
    constructor() {
        this.deferredPrompt = null;
        this.setupInstallPrompt();
    }
    
    setupInstallPrompt() {
        window.addEventListener('beforeinstallprompt', (e) => {
            e.preventDefault();
            this.deferredPrompt = e;
            this.showInstallButton();
        });
        
        window.addEventListener('appinstalled', () => {
            this.hideInstallButton();
            this.trackInstallation();
        });
    }
    
    async promptInstall() {
        if (!this.deferredPrompt) {
            return false;
        }
        
        this.deferredPrompt.prompt();
        const { outcome } = await this.deferredPrompt.userChoice;
        
        if (outcome === 'accepted') {
            console.log('User accepted the install prompt');
        } else {
            console.log('User dismissed the install prompt');
        }
        
        this.deferredPrompt = null;
        return outcome === 'accepted';
    }
    
    showInstallButton() {
        const installButton = document.getElementById('install-button');
        if (installButton) {
            installButton.style.display = 'block';
            installButton.addEventListener('click', () => this.promptInstall());
        }
    }
}
```

---

## 🔮 Future Architecture

### Planned Enhancements

#### 1. Multi-Language Support

```rust
// Future: Plugin-based language support
pub trait LanguageRuntime {
    fn execute(&mut self, code: &str) -> Result<ExecutionResult, RuntimeError>;
    fn get_completions(&self, context: &str) -> Vec<Completion>;
    fn validate_syntax(&self, code: &str) -> Result<(), SyntaxError>;
}

pub struct PythonRuntime {
    // PyO3 integration for Python code
}

impl LanguageRuntime for PythonRuntime {
    fn execute(&mut self, code: &str) -> Result<ExecutionResult, RuntimeError> {
        // Execute Python code via PyO3
        todo!()
    }
}
```

#### 2. Real-time Collaboration

```typescript
interface CollaborationMessage {
    type: 'cell_update' | 'cursor_move' | 'selection_change';
    userId: string;
    timestamp: number;
    data: {
        cellId?: string;
        content?: string;
        cursor?: { line: number, column: number };
        selection?: { start: Position, end: Position };
    };
}

class CollaborationEngine {
    private websocket: WebSocket;
    private operationalTransform: OTEngine;
    
    constructor(notebookId: string) {
        this.websocket = new WebSocket(`ws://server/notebook/${notebookId}`);
        this.operationalTransform = new OTEngine();
        this.setupMessageHandlers();
    }
    
    private handleRemoteOperation(op: Operation) {
        const transformedOp = this.operationalTransform.transform(op);
        this.applyOperation(transformedOp);
    }
}
```

#### 3. Advanced Visualization

```rust
// Future: Rich output rendering
pub enum OutputType {
    Text(String),
    Html(String),
    Image { data: Vec<u8>, mime_type: String },
    Plot { data: PlotData, config: PlotConfig },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Interactive { widget: Widget, state: WidgetState },
}

pub struct PlotData {
    series: Vec<Series>,
    layout: Layout,
    config: Config,
}
```

#### 4. Cloud Integration

```typescript
interface CloudProvider {
    saveNotebook(id: string, notebook: Notebook): Promise<void>;
    loadNotebook(id: string): Promise<Notebook>;
    shareNotebook(id: string, permissions: SharePermissions): Promise<ShareLink>;
    syncNotebook(id: string, changes: Change[]): Promise<SyncResult>;
}

class GitHubProvider implements CloudProvider {
    async saveNotebook(id: string, notebook: Notebook): Promise<void> {
        // Save to GitHub repository
        const content = JSON.stringify(notebook);
        await this.githubApi.createOrUpdateFile(id, content);
    }
}
```

### Performance Goals

**Version 2.0 Targets:**
- **Cold Start**: <100ms notebook initialization
- **Hot Execution**: <10ms cell execution for simple operations  
- **Memory**: <5MB base memory footprint
- **Bundle Size**: <100KB WASM module
- **Concurrent Cells**: Support 1000+ cells with virtual rendering
- **Real-time Sync**: <50ms collaboration latency

### Architecture Evolution

```
Current (v1.90.0)          Future (v2.0+)
┌─────────────────┐       ┌─────────────────────────┐
│ Single Language │  →    │ Multi-Language Runtime  │
│ Local Storage   │  →    │ Cloud + Local Hybrid   │
│ Single User     │  →    │ Real-time Collaboration │
│ Basic Output    │  →    │ Rich Visualizations     │
│ Manual Sharing  │  →    │ Integrated Sharing      │
└─────────────────┘       └─────────────────────────┘
```

---

*This architecture documentation covers Ruchy Notebook v1.90.0 design decisions and implementation patterns. For architectural changes and evolution, refer to the GitHub repository and design documents.*