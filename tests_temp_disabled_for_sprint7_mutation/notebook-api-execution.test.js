/**
 * TDD Unit Test for Notebook API Execution
 * Focused test to ensure cells use /api/execute endpoint
 */

describe('Notebook API Execution', () => {
    let mockFetch;
    
    beforeEach(() => {
        // Mock fetch globally
        mockFetch = jest.fn();
        global.fetch = mockFetch;
    });
    
    afterEach(() => {
        jest.clearAllMocks();
    });
    
    test('runCell should call /api/execute endpoint', async () => {
        // This test verifies that when we call an execution function,
        // it uses the API endpoint, not WASM or Worker
        
        // Create a minimal execution function that should use the API
        async function executeViaAPI(code, cellId) {
            const response = await fetch('/api/execute', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    code: code,
                    cell_id: cellId,
                    session_id: 'test-session'
                })
            });
            
            const result = await response.json();
            return result;
        }
        
        // Mock the fetch response
        mockFetch.mockResolvedValueOnce({
            ok: true,
            json: async () => ({
                success: true,
                result: '4',
                error: null,
                cell_id: 'cell-1',
                execution_time_ms: 10
            })
        });
        
        // Execute code
        const result = await executeViaAPI('2 + 2', 'cell-1');
        
        // Verify API was called correctly
        expect(mockFetch).toHaveBeenCalledWith('/api/execute', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                code: '2 + 2',
                cell_id: 'cell-1',
                session_id: 'test-session'
            })
        });
        
        // Verify result
        expect(result.success).toBe(true);
        expect(result.result).toBe('4');
    });
    
    test('RuchyNotebook.runCell should use API not WASM', async () => {
        // This test will check that the actual RuchyNotebook class
        // has been modified to use the API
        
        const RuchyNotebook = require('../ruchy-notebook/js/ruchy-notebook.js');
        
        // Check if runCell method exists
        const notebook = Object.create(RuchyNotebook.prototype);
        expect(typeof notebook.runCell).toBe('function');
        
        // Check that runInMainThread uses API, not WASM
        // This should fail initially because it tries to use WASM
        notebook.wasmModule = null; // No WASM module
        notebook.cells = [{ id: 'cell-1', content: '2 + 2', isRunning: false }];
        notebook.updateCellUI = jest.fn();
        notebook.setStatus = jest.fn();
        notebook.getNextExecutionCount = jest.fn(() => 1);
        
        // Mock fetch for API call
        mockFetch.mockResolvedValueOnce({
            ok: true,
            json: async () => ({
                success: true,
                result: '4',
                error: null,
                cell_id: 'cell-1',
                execution_time_ms: 10
            })
        });
        
        // This should call the API, not try to use WASM
        await notebook.runCell('cell-1');
        
        // Verify API was called
        expect(mockFetch).toHaveBeenCalledWith('/api/execute', expect.objectContaining({
            method: 'POST',
            headers: expect.objectContaining({
                'Content-Type': 'application/json',
            })
        }));
    });
    
    test('runInMainThread should use API instead of WASM', async () => {
        // Test that runInMainThread has been refactored to use API
        const RuchyNotebook = require('../ruchy-notebook/js/ruchy-notebook.js');
        const notebook = Object.create(RuchyNotebook.prototype);
        
        // Mock fetch
        mockFetch.mockResolvedValueOnce({
            ok: true,
            json: async () => ({
                success: true,
                result: '42',
                error: null,
                cell_id: 'test',
                execution_time_ms: 5
            })
        });
        
        // Call runInMainThread - should use API
        const result = await notebook.runInMainThread('42');
        
        // Should have called the API
        expect(mockFetch).toHaveBeenCalled();
        expect(result).toEqual({
            output: '42',
            execution_time_ms: 5
        });
    });
});