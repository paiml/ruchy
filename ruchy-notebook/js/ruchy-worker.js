/**
 * Ruchy Notebook WebWorker
 * Executes Ruchy code in a separate thread to prevent blocking the UI
 */

let wasmModule = null;
let notebook = null;

// Import WASM module
async function initWasm() {
    try {
        console.log('Worker: Loading WASM module...');
        
        // Dynamic import of the WASM module
        wasmModule = await import('./pkg/ruchy_notebook.js');
        await wasmModule.default();
        
        // Initialize notebook instance
        notebook = new wasmModule.WasmNotebook();
        
        console.log('Worker: WASM module loaded successfully');
        
        // Notify main thread that worker is ready
        postMessage({
            type: 'ready',
            success: true
        });
        
    } catch (error) {
        console.error('Worker: Failed to load WASM:', error);
        postMessage({
            type: 'ready',
            success: false,
            error: error.message
        });
    }
}

// Handle messages from main thread
self.addEventListener('message', async (e) => {
    const { id, type, code, timeout } = e.data;
    
    try {
        switch (type) {
            case 'init':
                await initWasm();
                break;
                
            case 'execute':
                if (!notebook) {
                    throw new Error('Notebook not initialized');
                }
                
                const startTime = performance.now();
                
                // Execute with optional timeout
                let result;
                if (timeout) {
                    result = await executeWithTimeout(code, timeout);
                } else {
                    result = notebook.execute(code);
                }
                
                const executionTime = performance.now() - startTime;
                
                postMessage({
                    id,
                    type: 'result',
                    success: true,
                    result: {
                        output: result.output,
                        success: result.success,
                        execution_time_ms: executionTime,
                        memory_used: result.memory_used
                    }
                });
                break;
                
            case 'reset':
                if (notebook) {
                    notebook.reset();
                }
                
                postMessage({
                    id,
                    type: 'reset',
                    success: true
                });
                break;
                
            case 'memory_info':
                const memoryUsage = notebook ? notebook.get_memory_usage() : 0;
                const runtimeMs = notebook ? notebook.get_runtime_ms() : 0;
                
                postMessage({
                    id,
                    type: 'memory_info',
                    success: true,
                    result: {
                        memory_usage: memoryUsage,
                        runtime_ms: runtimeMs
                    }
                });
                break;
                
            default:
                throw new Error(`Unknown message type: ${type}`);
        }
        
    } catch (error) {
        postMessage({
            id,
            type: 'error',
            success: false,
            error: error.message
        });
    }
});

// Execute code with timeout using AbortController-like pattern
async function executeWithTimeout(code, timeoutMs) {
    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
            reject(new Error(`Execution timeout after ${timeoutMs}ms`));
        }, timeoutMs);
        
        try {
            const result = notebook.execute(code);
            clearTimeout(timer);
            resolve(result);
        } catch (error) {
            clearTimeout(timer);
            reject(error);
        }
    });
}

// Error handling for uncaught errors
self.addEventListener('error', (error) => {
    console.error('Worker error:', error);
    postMessage({
        type: 'worker_error',
        success: false,
        error: error.message
    });
});

// Unhandled promise rejections
self.addEventListener('unhandledrejection', (event) => {
    console.error('Worker unhandled rejection:', event.reason);
    postMessage({
        type: 'worker_error', 
        success: false,
        error: event.reason
    });
});

// Initialize immediately when worker starts
initWasm();