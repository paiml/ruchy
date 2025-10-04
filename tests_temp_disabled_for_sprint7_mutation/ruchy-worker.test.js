/**
 * Tests for Ruchy Worker
 * Coverage target: >80%
 */

describe('RuchyWorker', () => {
    let worker;
    let postMessageSpy;
    let originalAddEventListener;

    beforeEach(() => {
        // Mock worker environment
        postMessageSpy = jest.fn();
        global.postMessage = postMessageSpy;
        
        // Mock addEventListener for worker
        originalAddEventListener = global.addEventListener;
        global.addEventListener = jest.fn((event, handler) => {
            if (event === 'message') {
                global.onmessage = handler;
            }
        });

        // Mock importScripts
        global.importScripts = jest.fn();
    });

    afterEach(() => {
        global.addEventListener = originalAddEventListener;
        delete global.onmessage;
        delete global.postMessage;
        delete global.importScripts;
    });

    describe('Message Handling', () => {
        test('should handle execute message', () => {
            // Simulate worker script
            require('../js/ruchy-worker.js');
            
            const message = {
                data: {
                    type: 'execute',
                    code: '1 + 1',
                    cellId: 'cell-123',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'result',
                    cellId: 'cell-123',
                })
            );
        });

        test('should handle init message', () => {
            require('../js/ruchy-worker.js');
            
            const message = {
                data: {
                    type: 'init',
                    wasmPath: './pkg/ruchy_bg.wasm',
                },
            };
            
            global.onmessage(message);
            
            expect(importScripts).toHaveBeenCalled();
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'ready',
                })
            );
        });

        test('should handle reset message', () => {
            require('../js/ruchy-worker.js');
            
            const message = {
                data: {
                    type: 'reset',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'reset-complete',
                })
            );
        });

        test('should handle unknown message type', () => {
            require('../js/ruchy-worker.js');
            
            const message = {
                data: {
                    type: 'unknown',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'error',
                    error: expect.stringContaining('Unknown message type'),
                })
            );
        });
    });

    describe('Code Execution', () => {
        beforeEach(() => {
            require('../js/ruchy-worker.js');
        });

        test('should execute valid code', () => {
            const message = {
                data: {
                    type: 'execute',
                    code: 'let x = 42; x',
                    cellId: 'test-cell',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'result',
                    success: true,
                    cellId: 'test-cell',
                })
            );
        });

        test('should handle execution errors', () => {
            const message = {
                data: {
                    type: 'execute',
                    code: 'throw new Error("Test error")',
                    cellId: 'error-cell',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'result',
                    success: false,
                    error: expect.stringContaining('Test error'),
                    cellId: 'error-cell',
                })
            );
        });

        test('should measure execution time', () => {
            const message = {
                data: {
                    type: 'execute',
                    code: '2 + 2',
                    cellId: 'timing-cell',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    executionTime: expect.any(Number),
                })
            );
        });

        test('should handle timeout', (done) => {
            const message = {
                data: {
                    type: 'execute',
                    code: 'while(true) {}',
                    cellId: 'timeout-cell',
                    timeout: 100,
                },
            };
            
            global.onmessage(message);
            
            setTimeout(() => {
                expect(postMessageSpy).toHaveBeenCalledWith(
                    expect.objectContaining({
                        type: 'result',
                        success: false,
                        error: expect.stringContaining('timeout'),
                    })
                );
                done();
            }, 150);
        });
    });

    describe('WASM Integration', () => {
        test('should load WASM module', () => {
            require('../js/ruchy-worker.js');
            
            const message = {
                data: {
                    type: 'init',
                    wasmPath: './pkg/ruchy_bg.wasm',
                },
            };
            
            global.onmessage(message);
            
            expect(importScripts).toHaveBeenCalledWith(
                expect.stringContaining('ruchy')
            );
        });

        test('should handle WASM loading error', () => {
            global.importScripts = jest.fn().mockImplementation(() => {
                throw new Error('Failed to load WASM');
            });
            
            require('../js/ruchy-worker.js');
            
            const message = {
                data: {
                    type: 'init',
                    wasmPath: './invalid.wasm',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'error',
                    error: expect.stringContaining('Failed to load WASM'),
                })
            );
        });
    });

    describe('Memory Management', () => {
        beforeEach(() => {
            require('../js/ruchy-worker.js');
        });

        test('should track memory usage', () => {
            const message = {
                data: {
                    type: 'memory-stats',
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'memory-stats',
                    used: expect.any(Number),
                    limit: expect.any(Number),
                })
            );
        });

        test('should clear memory on reset', () => {
            // Execute some code first
            global.onmessage({
                data: {
                    type: 'execute',
                    code: 'let bigArray = new Array(1000).fill(0)',
                    cellId: 'memory-test',
                },
            });
            
            // Reset
            global.onmessage({
                data: {
                    type: 'reset',
                },
            });
            
            // Check memory is cleared
            global.onmessage({
                data: {
                    type: 'memory-stats',
                },
            });
            
            const memoryCall = postMessageSpy.mock.calls.find(
                (call) => call[0].type === 'memory-stats'
            );
            
            expect(memoryCall[0].used).toBeLessThan(1000000);
        });
    });

    describe('Error Handling', () => {
        beforeEach(() => {
            require('../js/ruchy-worker.js');
        });

        test('should handle malformed messages', () => {
            const message = {
                data: null,
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'error',
                    error: expect.stringContaining('Invalid message'),
                })
            );
        });

        test('should handle missing required fields', () => {
            const message = {
                data: {
                    type: 'execute',
                    // Missing code and cellId
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'error',
                    error: expect.stringContaining('Missing required'),
                })
            );
        });

        test('should recover from crashes', () => {
            // Simulate crash
            const errorMessage = {
                data: {
                    type: 'execute',
                    code: 'crashingCode()',
                    cellId: 'crash-cell',
                },
            };
            
            // Mock crash
            const originalOnmessage = global.onmessage;
            global.onmessage = jest.fn().mockImplementation(() => {
                throw new Error('Worker crashed');
            });
            
            expect(() => global.onmessage(errorMessage)).toThrow();
            
            // Restore and verify recovery
            global.onmessage = originalOnmessage;
            
            const recoveryMessage = {
                data: {
                    type: 'execute',
                    code: '1 + 1',
                    cellId: 'recovery-cell',
                },
            };
            
            global.onmessage(recoveryMessage);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'result',
                    success: true,
                })
            );
        });
    });

    describe('Performance Monitoring', () => {
        beforeEach(() => {
            require('../js/ruchy-worker.js');
        });

        test('should track execution performance', () => {
            const message = {
                data: {
                    type: 'execute',
                    code: 'Math.sqrt(16)',
                    cellId: 'perf-cell',
                    measurePerformance: true,
                },
            };
            
            global.onmessage(message);
            
            expect(postMessageSpy).toHaveBeenCalledWith(
                expect.objectContaining({
                    performance: expect.objectContaining({
                        executionTime: expect.any(Number),
                        memoryUsed: expect.any(Number),
                    }),
                })
            );
        });

        test('should batch performance metrics', () => {
            const messages = [];
            for (let i = 0; i < 5; i++) {
                messages.push({
                    data: {
                        type: 'execute',
                        code: `${i} * 2`,
                        cellId: `batch-${i}`,
                        measurePerformance: true,
                    },
                });
            }
            
            messages.forEach((msg) => global.onmessage(msg));
            
            // Request aggregated metrics
            global.onmessage({
                data: {
                    type: 'get-metrics',
                },
            });
            
            const metricsCall = postMessageSpy.mock.calls.find(
                (call) => call[0].type === 'metrics'
            );
            
            expect(metricsCall[0].metrics).toHaveProperty('totalExecutions', 5);
            expect(metricsCall[0].metrics).toHaveProperty('averageTime');
        });
    });
});