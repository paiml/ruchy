/**
 * TDD Test Suite for Notebook Cell Execution
 * Tests that notebook cells actually execute code via API
 */

// Import directly
const RuchyNotebook = require('../ruchy-notebook/js/ruchy-notebook.js');

describe('Notebook Cell Execution', () => {
    let notebook;
    let mockFetch;
    
    beforeEach(() => {
        // Mock fetch API
        mockFetch = jest.fn();
        global.fetch = mockFetch;
        
        // Mock WebAssembly
        global.WebAssembly = {
            instantiate: jest.fn().mockResolvedValue({
                instance: { exports: {} }
            })
        };
        
        // Mock Worker
        global.Worker = jest.fn();
        
        // Create container
        document.body.innerHTML = '<div id="notebook-container"></div>';
        const container = document.getElementById('notebook-container');
        
        // Create notebook instance
        notebook = new RuchyNotebook(container, { useWorker: false });
    });
    
    afterEach(() => {
        jest.clearAllMocks();
    });
    
    describe('API Integration', () => {
        test('should call /api/execute endpoint when running a cell', async () => {
            // Arrange
            const cellId = 'cell-1';
            const code = '2 + 2';
            const expectedResponse = {
                success: true,
                result: '4',
                error: null,
                cell_id: cellId,
                execution_time_ms: 10
            };
            
            mockFetch.mockResolvedValueOnce({
                ok: true,
                json: async () => expectedResponse
            });
            
            // Add a cell with code
            notebook.addCell();
            const cell = notebook.cells[0];
            cell.content = code;
            cell.id = cellId;
            
            // Act
            await notebook.runCell(cellId);
            
            // Assert
            expect(mockFetch).toHaveBeenCalledWith('/api/execute', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    code: code,
                    cell_id: cellId,
                    session_id: expect.any(String)
                })
            });
        });
        
        test('should display result from API in cell output', async () => {
            // Arrange
            const cellId = 'cell-1';
            const code = '10 * 5';
            const expectedResult = '50';
            
            mockFetch.mockResolvedValueOnce({
                ok: true,
                json: async () => ({
                    success: true,
                    result: expectedResult,
                    error: null,
                    cell_id: cellId,
                    execution_time_ms: 5
                })
            });
            
            notebook.addCell();
            const cell = notebook.cells[0];
            cell.content = code;
            cell.id = cellId;
            
            // Act
            await notebook.runCell(cellId);
            
            // Assert
            expect(cell.output).toBe(expectedResult);
            expect(cell.executionCount).toBeDefined();
        });
        
        test('should handle API errors gracefully', async () => {
            // Arrange
            const cellId = 'cell-1';
            const code = 'invalid syntax';
            const errorMessage = 'Parse error: unexpected token';
            
            mockFetch.mockResolvedValueOnce({
                ok: true,
                json: async () => ({
                    success: false,
                    result: null,
                    error: errorMessage,
                    cell_id: cellId,
                    execution_time_ms: 2
                })
            });
            
            notebook.addCell();
            const cell = notebook.cells[0];
            cell.content = code;
            cell.id = cellId;
            
            // Act
            await notebook.runCell(cellId);
            
            // Assert
            expect(cell.output).toContain('Error');
            expect(cell.output).toContain(errorMessage);
        });
        
        test('should handle network errors', async () => {
            // Arrange
            const cellId = 'cell-1';
            const code = 'print("hello")';
            
            mockFetch.mockRejectedValueOnce(new Error('Network error'));
            
            notebook.addCell();
            const cell = notebook.cells[0];
            cell.content = code;
            cell.id = cellId;
            
            // Act
            await notebook.runCell(cellId);
            
            // Assert
            expect(cell.output).toContain('Error');
            expect(cell.output).toContain('Network error');
        });
        
        test('should prevent double execution of same cell', async () => {
            // Arrange
            const cellId = 'cell-1';
            const code = '42';
            
            mockFetch.mockImplementation(() => 
                new Promise(resolve => setTimeout(() => resolve({
                    ok: true,
                    json: async () => ({
                        success: true,
                        result: '42',
                        error: null,
                        cell_id: cellId,
                        execution_time_ms: 100
                    })
                }), 100))
            );
            
            notebook.addCell();
            const cell = notebook.cells[0];
            cell.content = code;
            cell.id = cellId;
            
            // Act - try to run the same cell twice
            const promise1 = notebook.runCell(cellId);
            const promise2 = notebook.runCell(cellId);
            
            await Promise.all([promise1, promise2]);
            
            // Assert - should only call API once
            expect(mockFetch).toHaveBeenCalledTimes(1);
        });
        
        test('should update UI status during execution', async () => {
            // Arrange
            const cellId = 'cell-1';
            const code = 'heavy_computation()';
            let statusUpdates = [];
            
            // Mock setStatus to track calls
            notebook.setStatus = jest.fn((status) => {
                statusUpdates.push(status);
            });
            
            mockFetch.mockResolvedValueOnce({
                ok: true,
                json: async () => ({
                    success: true,
                    result: 'Done',
                    error: null,
                    cell_id: cellId,
                    execution_time_ms: 150
                })
            });
            
            notebook.addCell();
            const cell = notebook.cells[0];
            cell.content = code;
            cell.id = cellId;
            
            // Act
            await notebook.runCell(cellId);
            
            // Assert
            expect(statusUpdates).toContain('Running...');
            expect(statusUpdates[statusUpdates.length - 1]).toContain('Completed');
        });
    });
    
    describe('Session Management', () => {
        test('should maintain session ID across multiple executions', async () => {
            // Arrange
            const code1 = 'x = 10';
            const code2 = 'x * 2';
            let sessionIds = [];
            
            mockFetch.mockImplementation((url, options) => {
                const body = JSON.parse(options.body);
                sessionIds.push(body.session_id);
                return Promise.resolve({
                    ok: true,
                    json: async () => ({
                        success: true,
                        result: 'OK',
                        error: null,
                        cell_id: body.cell_id,
                        execution_time_ms: 5
                    })
                });
            });
            
            // Act
            notebook.addCell();
            notebook.cells[0].content = code1;
            await notebook.runCell(notebook.cells[0].id);
            
            notebook.addCell();
            notebook.cells[1].content = code2;
            await notebook.runCell(notebook.cells[1].id);
            
            // Assert
            expect(sessionIds[0]).toBeDefined();
            expect(sessionIds[1]).toBeDefined();
            expect(sessionIds[0]).toBe(sessionIds[1]); // Same session
        });
    });
    
    describe('Execution Order', () => {
        test('should execute cells sequentially when running all', async () => {
            // Arrange
            const executionOrder = [];
            
            mockFetch.mockImplementation((url, options) => {
                const body = JSON.parse(options.body);
                executionOrder.push(body.cell_id);
                return Promise.resolve({
                    ok: true,
                    json: async () => ({
                        success: true,
                        result: 'OK',
                        error: null,
                        cell_id: body.cell_id,
                        execution_time_ms: 5
                    })
                });
            });
            
            // Add three cells
            notebook.addCell();
            notebook.addCell();
            notebook.addCell();
            
            notebook.cells[0].content = 'first';
            notebook.cells[1].content = 'second';
            notebook.cells[2].content = 'third';
            
            // Act
            await notebook.runAllCells();
            
            // Wait for all to complete
            await new Promise(resolve => setTimeout(resolve, 500));
            
            // Assert
            expect(executionOrder.length).toBe(3);
            expect(executionOrder[0]).toBe(notebook.cells[0].id);
            expect(executionOrder[1]).toBe(notebook.cells[1].id);
            expect(executionOrder[2]).toBe(notebook.cells[2].id);
        });
    });
});