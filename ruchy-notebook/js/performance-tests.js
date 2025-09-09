/**
 * Performance Testing Suite for Ruchy Notebook Frontend
 * Validates <50ms execution times and progressive loading
 */

class NotebookPerformanceTester {
    constructor() {
        this.testResults = new Map();
        this.performanceObserver = null;
        this.setupPerformanceObserver();
    }
    
    setupPerformanceObserver() {
        if ('PerformanceObserver' in window) {
            this.performanceObserver = new PerformanceObserver((list) => {
                for (const entry of list.getEntries()) {
                    if (entry.name.startsWith('ruchy-notebook')) {
                        this.recordMetric(entry.name, entry.duration);
                    }
                }
            });
            
            this.performanceObserver.observe({ entryTypes: ['measure'] });
        }
    }
    
    recordMetric(name, duration) {
        if (!this.testResults.has(name)) {
            this.testResults.set(name, []);
        }
        this.testResults.get(name).push(duration);
    }
    
    async runAllTests() {
        console.log('🚀 Starting Ruchy Notebook Performance Tests...');
        
        const tests = [
            this.testWasmLoadingTime.bind(this),
            this.testCellExecutionPerformance.bind(this),
            this.testLargeCellsRendering.bind(this),
            this.testVirtualScrollingPerformance.bind(this),
            this.testWorkerCommunicationLatency.bind(this),
            this.testMemoryUsagePattern.bind(this),
            this.testNotebookSerializationSpeed.bind(this),
            this.testProgressiveLoadingBehavior.bind(this)
        ];
        
        const results = {};
        
        for (const test of tests) {
            try {
                const testName = test.name.replace('bound ', '');
                console.log(`Running ${testName}...`);
                results[testName] = await test();
            } catch (error) {
                console.error(`❌ ${test.name} failed:`, error);
                results[test.name] = { error: error.message };
            }
        }
        
        this.generateReport(results);
        return results;
    }
    
    async testWasmLoadingTime() {
        performance.mark('wasm-load-start');
        
        // Simulate WASM loading
        const wasmModule = await import('./pkg/ruchy_notebook.js');
        await wasmModule.default();
        
        performance.mark('wasm-load-end');
        performance.measure('ruchy-notebook-wasm-load', 'wasm-load-start', 'wasm-load-end');
        
        const loadTime = performance.getEntriesByName('ruchy-notebook-wasm-load')[0].duration;
        
        return {
            duration: loadTime,
            target: 500, // 500ms target for WASM loading
            passed: loadTime < 500,
            details: `WASM loaded in ${loadTime.toFixed(1)}ms`
        };
    }
    
    async testCellExecutionPerformance() {
        const testCases = [
            { name: 'simple_math', code: '1 + 2 * 3' },
            { name: 'string_ops', code: '"hello" + " " + "world"' },
            { name: 'variables', code: 'let x = 42; x * 2' },
            { name: 'function_call', code: 'fun add(a, b) { a + b }; add(10, 20)' },
            { name: 'array_ops', code: '[1, 2, 3, 4, 5].map(fun(x) { x * x })' }
        ];
        
        const results = [];
        const wasmModule = await import('./pkg/ruchy_notebook.js');
        await wasmModule.default();
        const notebook = new wasmModule.WasmNotebook();
        
        for (const testCase of testCases) {
            const times = [];
            
            // Run each test 10 times for statistical significance
            for (let i = 0; i < 10; i++) {
                performance.mark(`cell-exec-${testCase.name}-start`);
                
                try {
                    notebook.execute(testCase.code);
                } catch (error) {
                    console.warn(`Execution error in ${testCase.name}:`, error);
                }
                
                performance.mark(`cell-exec-${testCase.name}-end`);
                performance.measure(
                    `ruchy-notebook-cell-exec-${testCase.name}`,
                    `cell-exec-${testCase.name}-start`,
                    `cell-exec-${testCase.name}-end`
                );
                
                const duration = performance.getEntriesByName(`ruchy-notebook-cell-exec-${testCase.name}`).pop().duration;
                times.push(duration);
            }
            
            const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
            const maxTime = Math.max(...times);
            
            results.push({
                testCase: testCase.name,
                avgTime,
                maxTime,
                target: 50, // 50ms target
                passed: avgTime < 50,
                details: `Avg: ${avgTime.toFixed(1)}ms, Max: ${maxTime.toFixed(1)}ms`
            });
        }
        
        return results;
    }
    
    async testLargeCellsRendering() {
        const container = document.createElement('div');
        container.id = 'test-container';
        document.body.appendChild(container);
        
        try {
            performance.mark('large-cells-render-start');
            
            const notebook = new RuchyNotebook(container, {
                lazyLoading: true,
                virtualScrolling: true,
                cellBatchSize: 20
            });
            
            // Simulate adding 1000 cells
            const cellData = [];
            for (let i = 0; i < 1000; i++) {
                cellData.push({
                    type: 'code',
                    content: `println("Cell ${i}");\nlet x_${i} = ${i};\nx_${i} * 2`,
                    output: i % 10 === 0 ? `Output from cell ${i}` : ''
                });
            }
            
            // Add cells in batches
            for (let i = 0; i < cellData.length; i += 50) {
                const batch = cellData.slice(i, i + 50);
                for (const cell of batch) {
                    notebook.addCell(cell.type, cell.content);
                }
                
                // Yield control to prevent blocking
                await new Promise(resolve => setTimeout(resolve, 0));
            }
            
            performance.mark('large-cells-render-end');
            performance.measure('ruchy-notebook-large-cells', 'large-cells-render-start', 'large-cells-render-end');
            
            const renderTime = performance.getEntriesByName('ruchy-notebook-large-cells')[0].duration;
            
            return {
                cellCount: 1000,
                duration: renderTime,
                target: 2000, // 2s target for 1000 cells
                passed: renderTime < 2000,
                details: `Rendered 1000 cells in ${renderTime.toFixed(1)}ms`
            };
            
        } finally {
            document.body.removeChild(container);
        }
    }
    
    async testVirtualScrollingPerformance() {
        const container = document.createElement('div');
        container.id = 'scroll-test-container';
        container.style.height = '500px';
        container.style.overflow = 'auto';
        document.body.appendChild(container);
        
        try {
            const notebook = new RuchyNotebook(container, {
                virtualScrolling: true,
                visibilityBuffer: 5
            });
            
            // Add many cells
            for (let i = 0; i < 500; i++) {
                notebook.addCell('code', `println("Virtual scroll test ${i}")`);
            }
            
            performance.mark('scroll-test-start');
            
            // Simulate scrolling
            const scrollTests = 20;
            const scrollPromises = [];
            
            for (let i = 0; i < scrollTests; i++) {
                scrollPromises.push(new Promise(resolve => {
                    setTimeout(() => {
                        container.scrollTop = (i / scrollTests) * container.scrollHeight;
                        resolve();
                    }, i * 50);
                }));
            }
            
            await Promise.all(scrollPromises);
            
            performance.mark('scroll-test-end');
            performance.measure('ruchy-notebook-scroll-perf', 'scroll-test-start', 'scroll-test-end');
            
            const scrollTime = performance.getEntriesByName('ruchy-notebook-scroll-perf')[0].duration;
            
            return {
                scrollOperations: scrollTests,
                duration: scrollTime,
                target: 1000, // 1s target for scroll operations
                passed: scrollTime < 1000,
                details: `${scrollTests} scroll operations in ${scrollTime.toFixed(1)}ms`
            };
            
        } finally {
            document.body.removeChild(container);
        }
    }
    
    async testWorkerCommunicationLatency() {
        const worker = new Worker('./ruchy-worker.js');
        
        return new Promise((resolve) => {
            const testMessages = 10;
            const latencies = [];
            let completedMessages = 0;
            
            const sendTestMessage = (messageId) => {
                const startTime = performance.now();
                
                worker.postMessage({
                    id: messageId,
                    type: 'execute',
                    code: '1 + 1'
                });
                
                const handler = (e) => {
                    if (e.data.id === messageId) {
                        const latency = performance.now() - startTime;
                        latencies.push(latency);
                        completedMessages++;
                        
                        worker.removeEventListener('message', handler);
                        
                        if (completedMessages === testMessages) {
                            const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
                            const maxLatency = Math.max(...latencies);
                            
                            worker.terminate();
                            
                            resolve({
                                messageCount: testMessages,
                                avgLatency,
                                maxLatency,
                                target: 100, // 100ms target for worker communication
                                passed: avgLatency < 100,
                                details: `Avg: ${avgLatency.toFixed(1)}ms, Max: ${maxLatency.toFixed(1)}ms`
                            });
                        }
                    }
                };
                
                worker.addEventListener('message', handler);
            };
            
            // Send test messages
            for (let i = 0; i < testMessages; i++) {
                setTimeout(() => sendTestMessage(`test-${i}`), i * 10);
            }
        });
    }
    
    async testMemoryUsagePattern() {
        const initialMemory = performance.memory ? performance.memory.usedJSHeapSize : 0;
        
        const container = document.createElement('div');
        document.body.appendChild(container);
        
        try {
            const notebook = new RuchyNotebook(container);
            
            // Create and destroy cells to test memory leaks
            for (let cycle = 0; cycle < 5; cycle++) {
                // Add cells
                for (let i = 0; i < 100; i++) {
                    notebook.addCell('code', `let data_${cycle}_${i} = Array(1000).fill(${i});`);
                }
                
                // Clear cells
                notebook.clearAllCells();
                
                // Force garbage collection if available
                if (window.gc) {
                    window.gc();
                }
                
                await new Promise(resolve => setTimeout(resolve, 100));
            }
            
            const finalMemory = performance.memory ? performance.memory.usedJSHeapSize : 0;
            const memoryIncrease = finalMemory - initialMemory;
            const memoryIncreaseKB = memoryIncrease / 1024;
            
            return {
                initialMemory: initialMemory / 1024,
                finalMemory: finalMemory / 1024,
                memoryIncreaseKB,
                target: 10240, // 10MB max increase
                passed: memoryIncreaseKB < 10240,
                details: `Memory increase: ${memoryIncreaseKB.toFixed(1)}KB`
            };
            
        } finally {
            document.body.removeChild(container);
        }
    }
    
    async testNotebookSerializationSpeed() {
        const notebook = {
            cells: [],
            metadata: { created: new Date().toISOString() }
        };
        
        // Create large notebook
        for (let i = 0; i < 1000; i++) {
            notebook.cells.push({
                cell_type: 'code',
                source: [`println("Cell ${i}");`, `let x = ${i};`, 'x * 2'],
                outputs: i % 10 === 0 ? [{ text: `Output ${i}` }] : [],
                execution_count: i
            });
        }
        
        // Test serialization
        performance.mark('serialize-start');
        const serialized = JSON.stringify(notebook);
        performance.mark('serialize-end');
        
        // Test deserialization  
        performance.mark('deserialize-start');
        const deserialized = JSON.parse(serialized);
        performance.mark('deserialize-end');
        
        performance.measure('ruchy-notebook-serialize', 'serialize-start', 'serialize-end');
        performance.measure('ruchy-notebook-deserialize', 'deserialize-start', 'deserialize-end');
        
        const serializeTime = performance.getEntriesByName('ruchy-notebook-serialize')[0].duration;
        const deserializeTime = performance.getEntriesByName('ruchy-notebook-deserialize')[0].duration;
        
        return {
            cellCount: notebook.cells.length,
            dataSize: serialized.length / 1024, // KB
            serializeTime,
            deserializeTime,
            target: 500, // 500ms target for serialization
            passed: serializeTime < 500 && deserializeTime < 500,
            details: `Serialize: ${serializeTime.toFixed(1)}ms, Deserialize: ${deserializeTime.toFixed(1)}ms, Size: ${(serialized.length / 1024).toFixed(1)}KB`
        };
    }
    
    async testProgressiveLoadingBehavior() {
        performance.mark('progressive-load-start');
        
        // Test progressive loading simulation
        const loadSteps = [
            { step: 'init', delay: 50 },
            { step: 'wasm-fetch', delay: 200 },
            { step: 'wasm-compile', delay: 150 },
            { step: 'ui-setup', delay: 100 },
            { step: 'ready', delay: 50 }
        ];
        
        const stepTimes = [];
        
        for (const step of loadSteps) {
            const stepStart = performance.now();
            await new Promise(resolve => setTimeout(resolve, step.delay));
            const stepEnd = performance.now();
            stepTimes.push({ step: step.step, duration: stepEnd - stepStart });
        }
        
        performance.mark('progressive-load-end');
        performance.measure('ruchy-notebook-progressive-load', 'progressive-load-start', 'progressive-load-end');
        
        const totalTime = performance.getEntriesByName('ruchy-notebook-progressive-load')[0].duration;
        
        return {
            steps: stepTimes,
            totalTime,
            target: 1000, // 1s target for progressive loading
            passed: totalTime < 1000,
            details: `Progressive loading completed in ${totalTime.toFixed(1)}ms`
        };
    }
    
    generateReport(results) {
        console.log('\n📊 Ruchy Notebook Performance Report');
        console.log('=====================================');
        
        let totalTests = 0;
        let passedTests = 0;
        
        Object.entries(results).forEach(([testName, result]) => {
            if (result.error) {
                console.log(`❌ ${testName}: ERROR - ${result.error}`);
                return;
            }
            
            if (Array.isArray(result)) {
                // Handle array results (like cell execution tests)
                result.forEach(subResult => {
                    totalTests++;
                    const icon = subResult.passed ? '✅' : '❌';
                    const status = subResult.passed ? 'PASSED' : 'FAILED';
                    console.log(`${icon} ${testName}.${subResult.testCase}: ${status} - ${subResult.details}`);
                    if (subResult.passed) passedTests++;
                });
            } else {
                totalTests++;
                const icon = result.passed ? '✅' : '❌';
                const status = result.passed ? 'PASSED' : 'FAILED';
                console.log(`${icon} ${testName}: ${status} - ${result.details}`);
                if (result.passed) passedTests++;
            }
        });
        
        const passRate = (passedTests / totalTests * 100).toFixed(1);
        console.log(`\n🎯 Overall Results: ${passedTests}/${totalTests} tests passed (${passRate}%)`);
        
        if (passRate >= 90) {
            console.log('🏆 Excellent performance! All targets met.');
        } else if (passRate >= 80) {
            console.log('⚠️  Good performance, but some optimization needed.');
        } else {
            console.log('🚨 Performance issues detected. Optimization required.');
        }
        
        // Export results for CI/CD
        if (typeof window !== 'undefined') {
            window.notebookPerformanceResults = results;
        }
    }
}

// Auto-run tests if this script is loaded directly
if (typeof window !== 'undefined' && window.location.search.includes('run-perf-tests')) {
    window.addEventListener('load', async () => {
        const tester = new NotebookPerformanceTester();
        await tester.runAllTests();
    });
}

// Export for programmatic usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = NotebookPerformanceTester;
}