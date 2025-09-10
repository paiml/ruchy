/**
 * Ruchy Notebook JavaScript Components
 * Progressive Web App for interactive Ruchy programming
 */

class RuchyNotebook {
    constructor(container, options = {}) {
        this.container = container;
        this.options = {
            theme: 'dark',
            autoSave: true,
            saveInterval: 5000,
            maxCellCount: 100,
            wasmPath: './pkg/ruchy_notebook_bg.wasm',
            lazyLoading: true,
            virtualScrolling: true,
            cellBatchSize: 10,
            visibilityBuffer: 5,
            ...options
        };
        
        this.cells = [];
        this.wasmModule = null;
        this.worker = null;
        this.nextCellId = 1;
        this.visibleCells = new Set();
        this.intersectionObserver = null;
        this.sessionId = `session-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        
        this.init();
    }
    
    async init() {
        try {
            await this.loadWasm();
            this.setupUI();
            this.setupKeyboardShortcuts();
            this.setupWorker();
            this.setupVirtualScrolling();
            this.loadFromStorage();
        } catch (error) {
            console.error('Failed to initialize Ruchy Notebook:', error);
            this.showError('Failed to initialize notebook');
        }
    }
    
    async loadWasm() {
        try {
            console.log('Loading WASM module...');
            
            // Progressive loading with detailed steps
            this.showLoadingIndicator('Initializing runtime...');
            
            // Step 1: Check if WASM is already loaded
            if (this.wasmModule) {
                console.log('WASM already loaded, skipping...');
                this.hideLoadingIndicator();
                return;
            }
            
            // Step 2: Progressive loading with chunks
            this.showLoadingIndicator('Downloading runtime (0%)...');
            
            // Use dynamic import with progress tracking
            const wasmModule = await this.loadWasmWithProgress();
            
            this.showLoadingIndicator('Initializing runtime (90%)...');
            await wasmModule.default();
            
            this.showLoadingIndicator('Runtime ready (100%)...');
            this.wasmModule = wasmModule;
            
            // Cache for subsequent loads
            if ('serviceWorker' in navigator) {
                this.setupServiceWorkerCaching();
            }
            
            this.hideLoadingIndicator();
            console.log('WASM module loaded successfully');
            
        } catch (error) {
            this.hideLoadingIndicator();
            throw new Error(`Failed to load WASM: ${error.message}`);
        }
    }
    
    async loadWasmWithProgress() {
        const progressCallback = (percent) => {
            this.showLoadingIndicator(`Downloading runtime (${percent}%)...`);
        };
        
        // Simulate progressive loading for large WASM files
        const startTime = performance.now();
        
        progressCallback(10);
        await new Promise(resolve => setTimeout(resolve, 100));
        
        progressCallback(30);
        const wasmModule = await import('./pkg/ruchy_notebook.js');
        
        progressCallback(60);
        await new Promise(resolve => setTimeout(resolve, 50));
        
        progressCallback(80);
        const loadTime = performance.now() - startTime;
        console.log(`WASM module loaded in ${loadTime.toFixed(1)}ms`);
        
        return wasmModule;
    }
    
    setupServiceWorkerCaching() {
        // Register service worker for WASM caching
        navigator.serviceWorker.register('./sw.js')
            .then(registration => {
                console.log('Service worker registered for WASM caching');
            })
            .catch(error => {
                console.warn('Service worker registration failed:', error);
            });
    }
    
    setupUI() {
        this.container.className = `ruchy-notebook theme-${this.options.theme}`;
        this.container.innerHTML = `
            <div class="notebook-header">
                <h1>Ruchy Notebook</h1>
                <div class="toolbar">
                    <button id="add-cell">+ Cell</button>
                    <button id="run-all">Run All</button>
                    <button id="clear-all">Clear</button>
                    <button id="save">Save</button>
                    <button id="export">Export</button>
                </div>
            </div>
            <div class="notebook-body" id="notebook-body">
                <div class="cells-container" id="cells-container">
                    <!-- Cells will be inserted here -->
                </div>
            </div>
            <div class="notebook-footer">
                <div class="status-bar" id="status-bar">Ready</div>
            </div>
            <div class="loading-overlay" id="loading-overlay" style="display: none;">
                <div class="spinner"></div>
                <div class="loading-text">Loading...</div>
            </div>
        `;
        
        // Setup event listeners
        this.container.querySelector('#add-cell').addEventListener('click', () => this.addCell());
        this.container.querySelector('#run-all').addEventListener('click', () => this.runAllCells());
        this.container.querySelector('#clear-all').addEventListener('click', () => this.clearAllCells());
        this.container.querySelector('#save').addEventListener('click', () => this.save());
        this.container.querySelector('#export').addEventListener('click', () => this.exportNotebook());
        
        // Add initial cell if none exist
        if (this.cells.length === 0) {
            this.addCell();
        }
    }
    
    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch (e.key) {
                    case 'Enter':
                        e.preventDefault();
                        this.runCurrentCell();
                        break;
                    case 's':
                        e.preventDefault();
                        this.save();
                        break;
                    case 'n':
                        e.preventDefault();
                        this.addCell();
                        break;
                }
            }
            
            if (e.shiftKey && e.key === 'Enter') {
                e.preventDefault();
                this.runCurrentCell();
            }
        });
    }
    
    setupWorker() {
        if (window.Worker && this.options.useWorker !== false) {
            try {
                this.worker = new Worker('./js/ruchy-worker.js');
                this.worker.onmessage = (e) => this.handleWorkerMessage(e);
                console.log('WebWorker initialized');
            } catch (error) {
                console.warn('WebWorker not available, falling back to main thread');
            }
        }
    }
    
    setupVirtualScrolling() {
        if (!this.options.virtualScrolling) return;
        
        // Setup Intersection Observer for lazy loading
        this.intersectionObserver = new IntersectionObserver(
            (entries) => this.handleIntersection(entries),
            {
                root: this.container.querySelector('#cells-container'),
                rootMargin: '100px',
                threshold: 0.1
            }
        );
        
        // Setup scroll-based virtual scrolling
        const cellsContainer = this.container.querySelector('#cells-container');
        if (cellsContainer) {
            cellsContainer.addEventListener('scroll', 
                this.throttle(() => this.updateVirtualScrolling(), 16)
            );
        }
    }
    
    handleIntersection(entries) {
        entries.forEach(entry => {
            const cellId = entry.target.dataset.cellId;
            
            if (entry.isIntersecting) {
                this.visibleCells.add(cellId);
                this.activateCell(cellId);
            } else {
                this.visibleCells.delete(cellId);
                this.deactivateCell(cellId);
            }
        });
    }
    
    activateCell(cellId) {
        const cellElement = document.getElementById(cellId);
        if (cellElement && !cellElement.classList.contains('active')) {
            cellElement.classList.add('active');
            
            // Lazy load cell content if needed
            if (cellElement.classList.contains('lazy')) {
                this.renderCellContent(cellId);
                cellElement.classList.remove('lazy');
            }
        }
    }
    
    deactivateCell(cellId) {
        const cellElement = document.getElementById(cellId);
        if (cellElement && cellElement.classList.contains('active')) {
            cellElement.classList.remove('active');
            
            // Optionally unload non-essential content for memory efficiency
            if (this.options.aggressiveMemoryManagement) {
                this.unloadCellContent(cellId);
            }
        }
    }
    
    updateVirtualScrolling() {
        const container = this.container.querySelector('#cells-container');
        if (!container) return;
        
        const containerHeight = container.clientHeight;
        const scrollTop = container.scrollTop;
        const cellHeight = 150; // Estimated cell height
        
        const startIndex = Math.floor(scrollTop / cellHeight);
        const endIndex = Math.min(
            this.cells.length - 1,
            startIndex + Math.ceil(containerHeight / cellHeight) + this.options.visibilityBuffer
        );
        
        // Show only visible cells plus buffer
        this.cells.forEach((cell, index) => {
            const cellElement = document.getElementById(cell.id);
            if (cellElement) {
                const shouldShow = index >= startIndex - this.options.visibilityBuffer && 
                                 index <= endIndex + this.options.visibilityBuffer;
                
                cellElement.style.display = shouldShow ? 'block' : 'none';
            }
        });
    }
    
    throttle(func, limit) {
        let inThrottle;
        return function() {
            const args = arguments;
            const context = this;
            if (!inThrottle) {
                func.apply(context, args);
                inThrottle = true;
                setTimeout(() => inThrottle = false, limit);
            }
        }
    }
    
    addCell(type = 'code', content = '', index = null) {
        const cell = {
            id: `cell-${this.nextCellId++}`,
            type,
            content,
            output: '',
            executionCount: null,
            isRunning: false
        };
        
        if (index === null) {
            this.cells.push(cell);
        } else {
            this.cells.splice(index, 0, cell);
        }
        
        this.renderCell(cell);
        this.focusCell(cell.id);
        
        if (this.options.autoSave) {
            this.scheduleAutoSave();
        }
        
        return cell;
    }
    
    renderCell(cell, lazy = false) {
        const cellElement = document.createElement('div');
        cellElement.className = `cell ${cell.type}-cell${lazy ? ' lazy' : ''}`;
        cellElement.id = cell.id;
        cellElement.dataset.cellId = cell.id;
        
        if (lazy && this.options.lazyLoading) {
            // Render placeholder for lazy loading
            cellElement.innerHTML = `
                <div class="cell-placeholder">
                    <div class="placeholder-toolbar">
                        <span class="placeholder-indicator">Loading...</span>
                        <span class="execution-count">[${cell.executionCount || ' '}]</span>
                    </div>
                    <div class="placeholder-content"></div>
                </div>
            `;
            
            // Observe for intersection
            if (this.intersectionObserver) {
                this.intersectionObserver.observe(cellElement);
            }
        } else {
            // Render full cell content
            this.renderCellContent(cell.id);
        }
        
        // Insert into DOM
        const container = this.container.querySelector('#cells-container');
        container.appendChild(cellElement);
    }
    
    renderCellContent(cellId) {
        const cell = this.cells.find(c => c.id === cellId);
        if (!cell) return;
        
        const cellElement = document.getElementById(cellId);
        if (!cellElement) return;
        
        cellElement.innerHTML = `
            <div class="cell-toolbar">
                <button class="run-cell" title="Run Cell (Shift+Enter)">▶</button>
                <button class="delete-cell" title="Delete Cell">×</button>
                <span class="execution-count">[${cell.executionCount || ' '}]</span>
            </div>
            <div class="cell-input">
                <textarea class="code-input" placeholder="Enter Ruchy code...">${cell.content}</textarea>
            </div>
            <div class="cell-output" style="display: ${cell.output ? 'block' : 'none'}">
                <pre class="output-content">${cell.output}</pre>
            </div>
        `;
        
        // Event listeners
        const textarea = cellElement.querySelector('.code-input');
        const runButton = cellElement.querySelector('.run-cell');
        const deleteButton = cellElement.querySelector('.delete-cell');
        
        textarea.addEventListener('input', (e) => {
            cell.content = e.target.value;
            this.scheduleAutoSave();
        });
        
        textarea.addEventListener('keydown', (e) => {
            if (e.shiftKey && e.key === 'Enter') {
                e.preventDefault();
                this.runCell(cell.id);
            }
        });
        
        runButton.addEventListener('click', () => this.runCell(cell.id));
        deleteButton.addEventListener('click', () => this.deleteCell(cell.id));
    }
    
    unloadCellContent(cellId) {
        const cellElement = document.getElementById(cellId);
        if (!cellElement) return;
        
        // Convert to placeholder to save memory
        cellElement.innerHTML = `
            <div class="cell-placeholder">
                <div class="placeholder-toolbar">
                    <span class="placeholder-indicator">Click to load</span>
                </div>
                <div class="placeholder-content"></div>
            </div>
        `;
        
        cellElement.classList.add('lazy');
        cellElement.addEventListener('click', () => {
            this.renderCellContent(cellId);
            cellElement.classList.remove('lazy');
        }, { once: true });
    }
    
    async runCell(cellId) {
        const cell = this.cells.find(c => c.id === cellId);
        if (!cell) return;
        
        if (cell.isRunning) {
            console.log('Cell is already running');
            return;
        }
        
        cell.isRunning = true;
        this.updateCellUI(cell);
        this.setStatus('Running...');
        
        try {
            let result;
            
            if (this.worker) {
                // Use WebWorker for execution
                result = await this.runInWorker(cell.content, cell.id);
            } else {
                // Run via API (main thread)
                result = await this.runInMainThread(cell.content, cell.id);
            }
            
            cell.output = result.output;
            cell.executionCount = this.getNextExecutionCount();
            
            this.updateCellUI(cell);
            this.setStatus(`Completed in ${result.execution_time_ms.toFixed(1)}ms`);
            
        } catch (error) {
            cell.output = `Error: ${error.message}`;
            this.updateCellUI(cell);
            this.setStatus('Error');
        } finally {
            cell.isRunning = false;
            this.updateCellUI(cell);
        }
    }
    
    async runInMainThread(code, cellId) {
        // Use server API instead of WASM
        try {
            const response = await fetch('/api/execute', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    code: code,
                    cell_id: cellId || `cell-${Date.now()}`,
                    session_id: this.sessionId || 'default-session'
                })
            });
            
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            
            const result = await response.json();
            
            if (result.success) {
                return {
                    output: result.result || '',
                    execution_time_ms: result.execution_time_ms || 0
                };
            } else {
                throw new Error(result.error || 'Execution failed');
            }
        } catch (error) {
            console.error('API execution error:', error);
            throw error;
        }
    }
    
    async runInWorker(code, cellId) {
        // For now, just use the API even in "worker" mode
        // In the future, we could actually run a worker that calls the API
        return this.runInMainThread(code, cellId);
    }
    
    updateCellUI(cell) {
        const cellElement = document.getElementById(cell.id);
        if (!cellElement) return;
        
        const outputDiv = cellElement.querySelector('.cell-output');
        const outputContent = cellElement.querySelector('.output-content');
        const executionCount = cellElement.querySelector('.execution-count');
        const runButton = cellElement.querySelector('.run-cell');
        
        if (cell.output) {
            outputContent.textContent = cell.output;
            outputDiv.style.display = 'block';
        } else {
            outputDiv.style.display = 'none';
        }
        
        executionCount.textContent = `[${cell.executionCount || ' '}]`;
        
        if (cell.isRunning) {
            runButton.textContent = '⏸';
            cellElement.classList.add('running');
        } else {
            runButton.textContent = '▶';
            cellElement.classList.remove('running');
        }
    }
    
    deleteCell(cellId) {
        const index = this.cells.findIndex(c => c.id === cellId);
        if (index === -1) return;
        
        this.cells.splice(index, 1);
        
        const cellElement = document.getElementById(cellId);
        if (cellElement) {
            cellElement.remove();
        }
        
        this.scheduleAutoSave();
    }
    
    runAllCells() {
        this.cells.forEach(cell => {
            if (cell.type === 'code') {
                setTimeout(() => this.runCell(cell.id), 100);
            }
        });
    }
    
    clearAllCells() {
        this.cells.forEach(cell => {
            cell.output = '';
            cell.executionCount = null;
            this.updateCellUI(cell);
        });
    }
    
    focusCell(cellId) {
        const cellElement = document.getElementById(cellId);
        if (cellElement) {
            const textarea = cellElement.querySelector('.code-input');
            textarea.focus();
        }
    }
    
    save() {
        const data = {
            cells: this.cells,
            metadata: {
                created: new Date().toISOString(),
                ruchy_version: '1.90.0',
                notebook_version: '1.0.0'
            }
        };
        
        localStorage.setItem('ruchy-notebook', JSON.stringify(data));
        this.setStatus('Saved');
    }
    
    loadFromStorage() {
        try {
            const saved = localStorage.getItem('ruchy-notebook');
            if (saved) {
                const data = JSON.parse(saved);
                this.cells = data.cells || [];
                
                this.cells.forEach(cell => this.renderCell(cell));
                
                if (this.cells.length > 0) {
                    this.focusCell(this.cells[0].id);
                }
            }
        } catch (error) {
            console.warn('Failed to load from storage:', error);
        }
    }
    
    exportNotebook() {
        const notebook = {
            cells: this.cells.map(cell => ({
                cell_type: cell.type === 'code' ? 'code' : 'markdown',
                source: cell.content.split('\n'),
                outputs: cell.output ? [{ text: cell.output }] : [],
                execution_count: cell.executionCount
            })),
            metadata: {
                kernelspec: {
                    display_name: 'Ruchy',
                    language: 'ruchy',
                    name: 'ruchy'
                }
            },
            nbformat: 4,
            nbformat_minor: 5
        };
        
        const blob = new Blob([JSON.stringify(notebook, null, 2)], {
            type: 'application/json'
        });
        
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'notebook.ipynb';
        a.click();
        
        URL.revokeObjectURL(url);
    }
    
    getNextExecutionCount() {
        return Math.max(0, ...this.cells.map(c => c.executionCount || 0)) + 1;
    }
    
    scheduleAutoSave() {
        if (this.autoSaveTimeout) {
            clearTimeout(this.autoSaveTimeout);
        }
        
        this.autoSaveTimeout = setTimeout(() => {
            this.save();
        }, this.options.saveInterval);
    }
    
    setStatus(message) {
        const statusBar = this.container.querySelector('#status-bar');
        if (statusBar) {
            statusBar.textContent = message;
            
            // Clear after 5 seconds
            setTimeout(() => {
                if (statusBar.textContent === message) {
                    statusBar.textContent = 'Ready';
                }
            }, 5000);
        }
    }
    
    showLoadingIndicator(text = 'Loading...') {
        const overlay = this.container.querySelector('#loading-overlay');
        const textElement = overlay.querySelector('.loading-text');
        if (textElement) textElement.textContent = text;
        overlay.style.display = 'flex';
    }
    
    hideLoadingIndicator() {
        const overlay = this.container.querySelector('#loading-overlay');
        overlay.style.display = 'none';
    }
    
    showError(message) {
        this.setStatus(`Error: ${message}`);
        console.error(message);
    }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    const container = document.getElementById('notebook-container');
    if (container) {
        window.notebook = new RuchyNotebook(container);
    }
});

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = RuchyNotebook;
}