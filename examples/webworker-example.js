/**
 * Example usage of Ruchy WASM WebWorker execution model
 * 
 * This demonstrates how to use Ruchy in web browsers with WebWorkers
 * for non-blocking parallel compilation and execution.
 */

// Import the Ruchy WASM module
import init, { RuchyWasm, WebWorkerRuntime } from './pkg/ruchy.js';

async function main() {
    // Initialize WASM module
    await init();
    
    // Create compiler instance
    const compiler = new RuchyWasm();
    
    console.log("🦀 Ruchy WebWorker Example");
    console.log(`Version: ${compiler.version()}`);
    
    // Get WebWorker capabilities
    const workerInfo = JSON.parse(compiler.get_webworker_info());
    console.log("WebWorker Support:", workerInfo);
    
    // Example 1: Async compilation (non-blocking)
    console.log("\n📝 Example 1: Async Compilation");
    const sourceCode = `
        fun fibonacci(n: Int) -> Int {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        let result = fibonacci(10)
    `;
    
    try {
        const result = await compiler.compile_async(sourceCode);
        console.log("✅ Compiled successfully:");
        console.log(result.substring(0, 200) + "...");
    } catch (error) {
        console.error("❌ Compilation failed:", error);
    }
    
    // Example 2: Parallel cell compilation
    console.log("\n🔄 Example 2: Parallel Cell Compilation");
    const cells = [
        "let x = 42",
        "let y = x * 2", 
        "fun double(n: Int) -> Int { n * 2 }",
        "let result = double(21)"
    ];
    
    try {
        const results = await compiler.compile_cells_parallel(cells);
        console.log("✅ Parallel compilation completed:");
        results.forEach((result, index) => {
            const parsed = JSON.parse(result);
            if (parsed.success) {
                console.log(`  Cell ${index + 1}: ✅ Success`);
            } else {
                console.log(`  Cell ${index + 1}: ❌ ${parsed.error}`);
            }
        });
    } catch (error) {
        console.error("❌ Parallel compilation failed:", error);
    }
    
    // Example 3: WebWorker Runtime with load balancing
    console.log("\n⚡ Example 3: WebWorker Runtime");
    const runtime = new WebWorkerRuntime(4); // 4 max workers
    
    console.log("Worker Status:", JSON.parse(runtime.get_worker_status()));
    
    // Execute task with worker pool
    const taskData = `
        fun calculate_pi(iterations: Int) -> Float {
            let sum = 0.0
            for i in 0..iterations {
                sum = sum + (4.0 * Math.pow(-1.0, i) / (2.0 * i + 1.0))
            }
            sum
        }
        
        let pi_approximation = calculate_pi(1000)
    `;
    
    try {
        const result = await runtime.execute_with_workers(taskData);
        const parsed = JSON.parse(result);
        console.log("✅ Worker execution completed:");
        console.log(`  Success: ${parsed.success}`);
        console.log(`  Execution Time: ${parsed.execution_time_ms}ms`);
        console.log(`  Worker ID: ${parsed.worker_id}`);
        
        if (parsed.success) {
            console.log("  Result:", parsed.result.substring(0, 100) + "...");
        } else {
            console.log("  Error:", parsed.error);
        }
    } catch (error) {
        console.error("❌ Worker execution failed:", error);
    }
    
    console.log("\n🎉 WebWorker example completed!");
}

// Run the example
main().catch(console.error);

/**
 * WebWorker Integration Guide:
 * 
 * 1. Load Balancing: The WebWorkerRuntime distributes tasks across available workers
 * 2. Non-blocking: All compilation happens asynchronously
 * 3. Parallel Processing: Multiple cells can compile simultaneously
 * 4. Resource Management: Worker pool prevents resource exhaustion
 * 5. Error Handling: Comprehensive error reporting with context
 * 
 * Browser Compatibility:
 * - Chrome/Edge: Full WebWorker support
 * - Firefox: Full WebWorker support  
 * - Safari: WebWorker support with some limitations
 * 
 * Performance Notes:
 * - Each worker has its own WASM instance
 * - Memory usage scales with number of workers
 * - Optimal worker count: Number of CPU cores
 * - WebWorker overhead: ~2-5ms per task
 */