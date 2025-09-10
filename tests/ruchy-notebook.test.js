/**
 * Tests for RuchyNotebook class
 * Coverage target: >80%
 */

// Mock the module before importing
jest.mock('../js/ruchy-notebook.js', () => {
    const actual = jest.requireActual('../js/ruchy-notebook.js');
    return actual;
});

// Import after mocking
const { RuchyNotebook, NotebookCell, CodeExecutor } = require('../js/ruchy-notebook.js');

describe('RuchyNotebook', () => {
    let container;
    let notebook;

    beforeEach(() => {
        // Create a container element
        container = document.createElement('div');
        container.id = 'notebook-container';
        document.body.appendChild(container);
    });

    afterEach(() => {
        if (notebook) {
            notebook.destroy?.();
        }
        document.body.innerHTML = '';
    });

    describe('Initialization', () => {
        test('should initialize with default options', () => {
            notebook = new RuchyNotebook(container);
            
            expect(notebook.container).toBe(container);
            expect(notebook.options.theme).toBe('dark');
            expect(notebook.options.autoSave).toBe(true);
            expect(notebook.options.maxCellCount).toBe(100);
            expect(notebook.cells).toEqual([]);
        });

        test('should merge custom options', () => {
            const customOptions = {
                theme: 'light',
                autoSave: false,
                maxCellCount: 50,
            };
            
            notebook = new RuchyNotebook(container, customOptions);
            
            expect(notebook.options.theme).toBe('light');
            expect(notebook.options.autoSave).toBe(false);
            expect(notebook.options.maxCellCount).toBe(50);
        });

        test('should load WASM module', async () => {
            notebook = new RuchyNotebook(container);
            await notebook.init();
            
            expect(global.fetch).toHaveBeenCalledWith(
                expect.stringContaining('.wasm')
            );
            expect(notebook.wasmModule).toBeTruthy();
        });

        test('should handle WASM loading failure', async () => {
            global.fetch.mockRejectedValueOnce(new Error('Network error'));
            
            notebook = new RuchyNotebook(container);
            await notebook.init();
            
            expect(console.error).toHaveBeenCalledWith(
                expect.stringContaining('Failed to initialize'),
                expect.any(Error)
            );
        });
    });

    describe('Cell Management', () => {
        beforeEach(() => {
            notebook = new RuchyNotebook(container);
        });

        test('should add a new cell', () => {
            const cell = notebook.addCell('code');
            
            expect(cell).toBeDefined();
            expect(cell.type).toBe('code');
            expect(cell.id).toBeDefined();
            expect(notebook.cells).toContain(cell);
        });

        test('should add markdown cell', () => {
            const cell = notebook.addCell('markdown');
            
            expect(cell.type).toBe('markdown');
            expect(cell.content).toBe('');
        });

        test('should remove a cell', () => {
            const cell = notebook.addCell('code');
            const cellId = cell.id;
            
            notebook.removeCell(cellId);
            
            expect(notebook.cells).not.toContain(cell);
            expect(notebook.getCellById(cellId)).toBeUndefined();
        });

        test('should not exceed max cell count', () => {
            notebook.options.maxCellCount = 2;
            
            notebook.addCell('code');
            notebook.addCell('code');
            const cell3 = notebook.addCell('code');
            
            expect(cell3).toBeNull();
            expect(notebook.cells.length).toBe(2);
        });

        test('should update cell content', () => {
            const cell = notebook.addCell('code');
            const newContent = 'println("Hello, Ruchy!")';
            
            notebook.updateCell(cell.id, newContent);
            
            expect(cell.content).toBe(newContent);
        });

        test('should move cell up', () => {
            const cell1 = notebook.addCell('code');
            const cell2 = notebook.addCell('code');
            
            notebook.moveCellUp(cell2.id);
            
            expect(notebook.cells[0]).toBe(cell2);
            expect(notebook.cells[1]).toBe(cell1);
        });

        test('should move cell down', () => {
            const cell1 = notebook.addCell('code');
            const cell2 = notebook.addCell('code');
            
            notebook.moveCellDown(cell1.id);
            
            expect(notebook.cells[0]).toBe(cell2);
            expect(notebook.cells[1]).toBe(cell1);
        });

        test('should clear all cells', () => {
            notebook.addCell('code');
            notebook.addCell('markdown');
            notebook.addCell('code');
            
            notebook.clearAllCells();
            
            expect(notebook.cells).toEqual([]);
        });
    });

    describe('Code Execution', () => {
        beforeEach(async () => {
            notebook = new RuchyNotebook(container);
            await notebook.init();
        });

        test('should execute code cell', async () => {
            const cell = notebook.addCell('code');
            cell.content = 'let x = 42; x';
            
            const result = await notebook.executeCell(cell.id);
            
            expect(result).toBeDefined();
            expect(cell.output).toBeDefined();
            expect(cell.executionCount).toBeGreaterThan(0);
        });

        test('should handle execution errors', async () => {
            const cell = notebook.addCell('code');
            cell.content = 'invalid syntax {';
            
            // Mock error response
            notebook.wasmModule.execute_code = jest.fn().mockImplementation(() => {
                throw new Error('Syntax error');
            });
            
            const result = await notebook.executeCell(cell.id);
            
            expect(result.error).toBeDefined();
            expect(cell.output.error).toBeTruthy();
        });

        test('should execute all cells', async () => {
            const cell1 = notebook.addCell('code');
            const cell2 = notebook.addCell('code');
            cell1.content = '1 + 1';
            cell2.content = '2 + 2';
            
            await notebook.executeAll();
            
            expect(cell1.executionCount).toBeGreaterThan(0);
            expect(cell2.executionCount).toBeGreaterThan(0);
        });

        test('should skip markdown cells during execution', async () => {
            const codeCell = notebook.addCell('code');
            const markdownCell = notebook.addCell('markdown');
            codeCell.content = '42';
            markdownCell.content = '# Heading';
            
            await notebook.executeAll();
            
            expect(codeCell.executionCount).toBeGreaterThan(0);
            expect(markdownCell.executionCount).toBeUndefined();
        });

        test('should interrupt execution', async () => {
            const cell = notebook.addCell('code');
            cell.content = 'while(true) {}';
            
            const executionPromise = notebook.executeCell(cell.id);
            notebook.interruptExecution();
            
            const result = await executionPromise;
            
            expect(result.interrupted).toBe(true);
        });
    });

    describe('Storage and Persistence', () => {
        beforeEach(() => {
            notebook = new RuchyNotebook(container);
        });

        test('should save to localStorage', () => {
            const cell = notebook.addCell('code');
            cell.content = 'test content';
            
            notebook.save();
            
            expect(localStorage.setItem).toHaveBeenCalledWith(
                'ruchy-notebook',
                expect.stringContaining('test content')
            );
        });

        test('should load from localStorage', () => {
            const savedData = {
                cells: [
                    { id: 'cell1', type: 'code', content: 'loaded content' },
                ],
            };
            localStorage.getItem.mockReturnValueOnce(JSON.stringify(savedData));
            
            notebook.load();
            
            expect(notebook.cells.length).toBe(1);
            expect(notebook.cells[0].content).toBe('loaded content');
        });

        test('should auto-save if enabled', (done) => {
            notebook.options.autoSave = true;
            notebook.options.saveInterval = 100;
            
            notebook.addCell('code');
            notebook.startAutoSave();
            
            setTimeout(() => {
                expect(localStorage.setItem).toHaveBeenCalled();
                notebook.stopAutoSave();
                done();
            }, 150);
        });

        test('should export notebook as JSON', () => {
            const cell = notebook.addCell('code');
            cell.content = 'export test';
            
            const exported = notebook.exportNotebook();
            
            expect(exported).toBeDefined();
            expect(exported.cells).toHaveLength(1);
            expect(exported.metadata).toBeDefined();
        });

        test('should import notebook from JSON', () => {
            const notebookData = {
                cells: [
                    { type: 'code', content: 'imported' },
                    { type: 'markdown', content: '# Title' },
                ],
                metadata: { version: '1.0' },
            };
            
            notebook.importNotebook(notebookData);
            
            expect(notebook.cells).toHaveLength(2);
            expect(notebook.cells[0].content).toBe('imported');
        });
    });

    describe('UI and Interactions', () => {
        beforeEach(() => {
            notebook = new RuchyNotebook(container);
            notebook.setupUI();
        });

        test('should create toolbar', () => {
            const toolbar = container.querySelector('.notebook-toolbar');
            expect(toolbar).toBeTruthy();
        });

        test('should create cell container', () => {
            const cellContainer = container.querySelector('.cell-container');
            expect(cellContainer).toBeTruthy();
        });

        test('should handle keyboard shortcuts', () => {
            const cell = notebook.addCell('code');
            notebook.selectCell(cell.id);
            
            // Simulate Ctrl+Enter
            const event = new KeyboardEvent('keydown', {
                key: 'Enter',
                ctrlKey: true,
            });
            document.dispatchEvent(event);
            
            // Execution should be triggered
            expect(notebook.executeCell).toHaveBeenCalledWith(cell.id);
        });

        test('should handle cell selection', () => {
            const cell1 = notebook.addCell('code');
            const cell2 = notebook.addCell('code');
            
            notebook.selectCell(cell2.id);
            
            expect(notebook.selectedCellId).toBe(cell2.id);
            expect(cell2.element?.classList.contains('selected')).toBe(true);
        });

        test('should render cell output', () => {
            const cell = notebook.addCell('code');
            cell.output = { value: '42', type: 'text' };
            
            notebook.renderCellOutput(cell);
            
            const outputElement = cell.element?.querySelector('.cell-output');
            expect(outputElement?.textContent).toContain('42');
        });
    });

    describe('Virtual Scrolling', () => {
        beforeEach(() => {
            notebook = new RuchyNotebook(container, {
                virtualScrolling: true,
                visibilityBuffer: 2,
            });
        });

        test('should setup intersection observer', () => {
            notebook.setupVirtualScrolling();
            
            expect(notebook.intersectionObserver).toBeTruthy();
        });

        test('should track visible cells', () => {
            const cell1 = notebook.addCell('code');
            const cell2 = notebook.addCell('code');
            
            notebook.markCellVisible(cell1.id);
            
            expect(notebook.visibleCells.has(cell1.id)).toBe(true);
            expect(notebook.visibleCells.has(cell2.id)).toBe(false);
        });

        test('should lazy render cells', () => {
            // Add many cells
            for (let i = 0; i < 20; i++) {
                notebook.addCell('code');
            }
            
            // Only visible cells should be fully rendered
            const renderedCells = notebook.cells.filter(
                (cell) => cell.isRendered
            );
            
            expect(renderedCells.length).toBeLessThan(notebook.cells.length);
        });
    });

    describe('Worker Integration', () => {
        beforeEach(() => {
            notebook = new RuchyNotebook(container);
        });

        test('should setup worker', () => {
            notebook.setupWorker();
            
            expect(notebook.worker).toBeTruthy();
            expect(notebook.worker.onmessage).toBeDefined();
        });

        test('should execute code in worker', async () => {
            notebook.setupWorker();
            
            const result = await notebook.executeInWorker('1 + 1');
            
            expect(result).toBeDefined();
            expect(notebook.worker.postMessage).toHaveBeenCalled();
        });

        test('should handle worker errors', async () => {
            notebook.setupWorker();
            
            // Mock worker error
            notebook.worker.postMessage = jest.fn().mockImplementation(() => {
                if (notebook.worker.onerror) {
                    notebook.worker.onerror(new Error('Worker crashed'));
                }
            });
            
            await expect(
                notebook.executeInWorker('test')
            ).rejects.toThrow('Worker crashed');
        });

        test('should restart worker on crash', () => {
            notebook.setupWorker();
            const originalWorker = notebook.worker;
            
            notebook.restartWorker();
            
            expect(notebook.worker).not.toBe(originalWorker);
            expect(originalWorker.terminate).toHaveBeenCalled();
        });
    });

    describe('Error Handling', () => {
        beforeEach(() => {
            notebook = new RuchyNotebook(container);
        });

        test('should handle invalid cell ID', () => {
            const result = notebook.removeCell('invalid-id');
            
            expect(result).toBe(false);
            expect(console.error).toHaveBeenCalledWith(
                expect.stringContaining('Cell not found')
            );
        });

        test('should validate cell type', () => {
            const cell = notebook.addCell('invalid-type');
            
            expect(cell).toBeNull();
            expect(console.error).toHaveBeenCalledWith(
                expect.stringContaining('Invalid cell type')
            );
        });

        test('should handle corrupted storage data', () => {
            localStorage.getItem.mockReturnValueOnce('invalid json {');
            
            notebook.load();
            
            expect(console.error).toHaveBeenCalledWith(
                expect.stringContaining('Failed to load'),
                expect.any(Error)
            );
            expect(notebook.cells).toEqual([]);
        });

        test('should recover from WASM crash', async () => {
            notebook.wasmModule = null;
            
            const result = await notebook.executeCell('cell-id');
            
            expect(result.error).toContain('WASM module not loaded');
        });
    });

    describe('Performance', () => {
        beforeEach(() => {
            notebook = new RuchyNotebook(container);
        });

        test('should batch cell operations', () => {
            const cells = [];
            for (let i = 0; i < 10; i++) {
                cells.push({ type: 'code', content: `cell ${i}` });
            }
            
            notebook.addCellsBatch(cells);
            
            expect(notebook.cells.length).toBe(10);
            // Should trigger single render
            expect(notebook.render).toHaveBeenCalledTimes(1);
        });

        test('should throttle saves', (done) => {
            notebook.options.saveInterval = 100;
            
            // Trigger multiple saves
            for (let i = 0; i < 5; i++) {
                notebook.save();
            }
            
            setTimeout(() => {
                // Should only save once due to throttling
                expect(localStorage.setItem).toHaveBeenCalledTimes(1);
                done();
            }, 150);
        });

        test('should measure execution time', async () => {
            const cell = notebook.addCell('code');
            cell.content = '42';
            
            const result = await notebook.executeCell(cell.id);
            
            expect(result.executionTime).toBeDefined();
            expect(result.executionTime).toBeGreaterThanOrEqual(0);
        });
    });
});

describe('NotebookCell', () => {
    test('should create cell with unique ID', () => {
        const cell1 = new NotebookCell('code');
        const cell2 = new NotebookCell('code');
        
        expect(cell1.id).toBeDefined();
        expect(cell2.id).toBeDefined();
        expect(cell1.id).not.toBe(cell2.id);
    });

    test('should initialize with correct type', () => {
        const codeCell = new NotebookCell('code');
        const markdownCell = new NotebookCell('markdown');
        
        expect(codeCell.type).toBe('code');
        expect(markdownCell.type).toBe('markdown');
    });

    test('should track execution state', () => {
        const cell = new NotebookCell('code');
        
        expect(cell.isExecuting).toBe(false);
        
        cell.setExecuting(true);
        expect(cell.isExecuting).toBe(true);
        
        cell.setExecuting(false);
        expect(cell.isExecuting).toBe(false);
    });

    test('should store output', () => {
        const cell = new NotebookCell('code');
        const output = { value: '42', type: 'text' };
        
        cell.setOutput(output);
        
        expect(cell.output).toEqual(output);
    });

    test('should clear output', () => {
        const cell = new NotebookCell('code');
        cell.setOutput({ value: 'test' });
        
        cell.clearOutput();
        
        expect(cell.output).toBeNull();
    });
});

describe('CodeExecutor', () => {
    let executor;

    beforeEach(() => {
        executor = new CodeExecutor();
    });

    test('should parse and execute simple expressions', async () => {
        const result = await executor.execute('1 + 1');
        
        expect(result.success).toBe(true);
        expect(result.value).toBe('2');
    });

    test('should handle syntax errors', async () => {
        const result = await executor.execute('invalid {');
        
        expect(result.success).toBe(false);
        expect(result.error).toContain('Syntax error');
    });

    test('should maintain execution context', async () => {
        await executor.execute('let x = 10');
        const result = await executor.execute('x + 5');
        
        expect(result.value).toBe('15');
    });

    test('should reset context', async () => {
        await executor.execute('let x = 10');
        executor.reset();
        
        const result = await executor.execute('x');
        
        expect(result.success).toBe(false);
        expect(result.error).toContain('undefined');
    });
});