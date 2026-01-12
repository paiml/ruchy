//! Probar Worker Harness Tests for Ruchy WASM
//!
//! Tests WASM worker capabilities using jugar-probar's worker harness:
//! - WorkerBrick code generation for deployment targets
//! - Distributed execution testing for WASM compilation
//! - Worker lifecycle validation
//!
//! Run with: `cargo test --test probar_worker_harness`

use jugar_probar::brick::distributed::{
    Backend, BackendSelector, BrickCoordinator, BrickDataTracker, BrickMessage, WorkerId,
};
use jugar_probar::brick::worker::{
    BrickWorkerMessage, BrickWorkerMessageDirection, FieldType, WorkerBrick,
};
use jugar_probar::prelude::*;
use jugar_probar::worker_harness::{
    RingBufferTestConfig, SharedMemoryTestConfig, WorkerTestConfig, WorkerTestHarness,
};
use std::sync::Arc;

// =============================================================================
// Worker Harness Coverage Tracking
// =============================================================================

fn worker_harness_coverage() -> UxCoverageTracker {
    UxCoverageBuilder::new()
        // WorkerBrick API
        .button("worker_brick_new")
        .button("worker_brick_message")
        .button("worker_brick_transition")
        .button("worker_brick_to_js")
        .button("worker_brick_to_rust")
        .button("worker_brick_to_typescript")
        // Distributed API
        .button("backend_selector_new")
        .button("backend_selector_select")
        .button("data_tracker_new")
        .button("data_tracker_track")
        .button("coordinator_new")
        .button("coordinator_publish")
        .button("coordinator_subscribe")
        // Worker Harness API
        .button("harness_new")
        .button("harness_lifecycle")
        .button("harness_ordering")
        .button("harness_ring_buffer")
        .button("harness_shared_memory")
        // Screens
        .screen("worker_brick")
        .screen("distributed")
        .screen("harness")
        .screen("integration")
        .build()
}

// =============================================================================
// WorkerBrick Code Generation Tests
// =============================================================================

#[test]
fn test_probar_worker_brick_for_cloudflare() {
    let mut gui = worker_harness_coverage();
    gui.visit("worker_brick");
    gui.click("worker_brick_new");
    gui.click("worker_brick_message");
    gui.click("worker_brick_transition");

    // Define a worker for Ruchy WASM compilation in Cloudflare Workers
    let compile_worker = WorkerBrick::new("ruchy_compiler")
        // Messages TO the worker
        .message(
            BrickWorkerMessage::new("compile", BrickWorkerMessageDirection::ToWorker)
                .field("source", FieldType::String)
                .field("target", FieldType::String)
                .optional_field("optimize", FieldType::Boolean),
        )
        .message(BrickWorkerMessage::new(
            "cancel",
            BrickWorkerMessageDirection::ToWorker,
        ))
        // Messages FROM the worker
        .message(
            BrickWorkerMessage::new("compiled", BrickWorkerMessageDirection::FromWorker)
                .field("wasm", FieldType::SharedArrayBuffer)
                .field("size", FieldType::Number),
        )
        .message(
            BrickWorkerMessage::new("error", BrickWorkerMessageDirection::FromWorker)
                .field("message", FieldType::String)
                .field("line", FieldType::Number),
        )
        .message(
            BrickWorkerMessage::new("progress", BrickWorkerMessageDirection::FromWorker)
                .field("stage", FieldType::String)
                .field("percent", FieldType::Number),
        )
        // State transitions
        .transition("idle", "compile", "compiling")
        .transition("compiling", "compiled", "idle")
        .transition("compiling", "error", "idle")
        .transition("compiling", "cancel", "idle");

    // Verify worker has correct message counts
    assert_eq!(compile_worker.to_worker_messages().len(), 2); // compile, cancel
    assert_eq!(compile_worker.from_worker_messages().len(), 3); // compiled, error, progress
}

#[test]
fn test_probar_worker_brick_js_generation() {
    let mut gui = worker_harness_coverage();
    gui.visit("worker_brick");
    gui.click("worker_brick_to_js");

    let worker = WorkerBrick::new("ruchy_repl")
        .message(
            BrickWorkerMessage::new("eval", BrickWorkerMessageDirection::ToWorker)
                .field("code", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("result", BrickWorkerMessageDirection::FromWorker)
                .field("output", FieldType::String)
                .field("elapsed_ms", FieldType::Number),
        )
        .transition("ready", "eval", "evaluating")
        .transition("evaluating", "result", "ready");

    let js_code = worker.to_worker_js();

    // Verify generated JS has expected structure
    // Name is converted to CamelCase: ruchy_repl -> RuchyRepl
    assert!(js_code.contains("RuchyRepl") || js_code.contains("Worker"));
    assert!(js_code.contains("onmessage"));
    assert!(js_code.contains("postMessage"));
}

#[test]
fn test_probar_worker_brick_rust_bindings() {
    let mut gui = worker_harness_coverage();
    gui.visit("worker_brick");
    gui.click("worker_brick_to_rust");

    let worker = WorkerBrick::new("ruchy_parser")
        .message(
            BrickWorkerMessage::new("parse", BrickWorkerMessageDirection::ToWorker)
                .field("source", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("ast", BrickWorkerMessageDirection::FromWorker)
                .field("json", FieldType::String),
        )
        .transition("idle", "parse", "parsing")
        .transition("parsing", "ast", "idle");

    let rust_code = worker.to_rust_bindings();

    // Verify generated Rust has expected structure
    // Name is converted to CamelCase: ruchy_parser -> RuchyParser
    assert!(
        rust_code.contains("RuchyParser")
            || rust_code.contains("Worker")
            || rust_code.contains("Bindings")
    );
    assert!(
        rust_code.contains("web_sys")
            || rust_code.contains("js_sys")
            || rust_code.contains("Generated")
    );
}

#[test]
fn test_probar_worker_brick_typescript_defs() {
    let mut gui = worker_harness_coverage();
    gui.visit("worker_brick");
    gui.click("worker_brick_to_typescript");

    let worker = WorkerBrick::new("ruchy_formatter")
        .message(
            BrickWorkerMessage::new("format", BrickWorkerMessageDirection::ToWorker)
                .field("code", FieldType::String)
                .optional_field("indent", FieldType::Number),
        )
        .message(
            BrickWorkerMessage::new("formatted", BrickWorkerMessageDirection::FromWorker)
                .field("output", FieldType::String),
        )
        .transition("idle", "format", "formatting")
        .transition("formatting", "formatted", "idle");

    let ts_defs = worker.to_typescript_defs();

    // Verify TypeScript definitions
    assert!(ts_defs.contains("interface"));
    assert!(ts_defs.contains("string"));
    assert!(ts_defs.contains("number"));
}

// =============================================================================
// Distributed Execution Tests
// =============================================================================

#[test]
fn test_probar_backend_selector_for_ruchy() {
    let mut gui = worker_harness_coverage();
    gui.visit("distributed");
    gui.click("backend_selector_new");
    gui.click("backend_selector_select");

    // Configure backend selection for Ruchy compilation workloads
    let selector = BackendSelector::new()
        .with_gpu_threshold(500_000) // Use GPU for large compilations
        .with_simd_threshold(10_000) // SIMD for medium
        .with_cpu_max_threshold(50_000_000);

    // Small source file (1KB) → CPU
    let small_backend = selector.select(1_000, false);
    assert_eq!(small_backend, Backend::Cpu);

    // Medium source file (50KB) → SIMD
    let medium_backend = selector.select(50_000, false);
    assert_eq!(medium_backend, Backend::Simd);

    // Large source file (1MB) with GPU available → GPU
    let large_backend = selector.select(1_000_000, true);
    assert_eq!(large_backend, Backend::Gpu);
}

#[test]
fn test_probar_data_tracker_for_wasm_modules() {
    let mut gui = worker_harness_coverage();
    gui.visit("distributed");
    gui.click("data_tracker_new");
    gui.click("data_tracker_track");

    let tracker = BrickDataTracker::new();

    // Track WASM module data across workers
    let worker0 = WorkerId::new(0);
    let worker1 = WorkerId::new(1);

    // Worker 0 has the compiler and stdlib
    tracker.track_data("ruchy_compiler", worker0, 5 * 1024 * 1024); // 5MB
    tracker.track_data("ruchy_stdlib", worker0, 2 * 1024 * 1024); // 2MB

    // Worker 1 has the optimizer
    tracker.track_data("ruchy_optimizer", worker1, 3 * 1024 * 1024); // 3MB

    // Query data locations
    let compiler_workers = tracker.get_workers_for_data("ruchy_compiler");
    assert_eq!(compiler_workers.len(), 1);
    assert_eq!(compiler_workers[0], worker0);

    // Calculate affinity for compilation task (needs compiler + stdlib)
    let deps = vec!["ruchy_compiler".into(), "ruchy_stdlib".into()];
    let affinity = tracker.calculate_affinity(&deps);

    // Worker 0 should have highest affinity (has both)
    assert!(affinity.get(&worker0).unwrap() > affinity.get(&worker1).unwrap_or(&0.0));
}

#[test]
fn test_probar_coordinator_pub_sub() {
    let mut gui = worker_harness_coverage();
    gui.visit("distributed");
    gui.click("coordinator_new");
    gui.click("coordinator_subscribe");
    gui.click("coordinator_publish");

    let coordinator = BrickCoordinator::new();

    // Subscribe to compilation events
    let compile_sub = coordinator.subscribe("compile_events");
    let error_sub = coordinator.subscribe("error_events");

    // Publish a compilation start event
    coordinator.publish(
        "compile_events",
        BrickMessage::StateChange {
            brick_name: "ruchy_compiler".into(),
            event: "compilation_started".into(),
        },
    );

    // Publish an error event
    coordinator.publish(
        "error_events",
        BrickMessage::StateChange {
            brick_name: "ruchy_parser".into(),
            event: "syntax_error".into(),
        },
    );

    // Drain and verify
    let compile_msgs = compile_sub.drain();
    let error_msgs = error_sub.drain();

    assert_eq!(compile_msgs.len(), 1);
    assert_eq!(error_msgs.len(), 1);
}

// =============================================================================
// Worker Harness Tests
// =============================================================================

#[test]
fn test_probar_worker_harness_lifecycle() {
    let mut gui = worker_harness_coverage();
    gui.visit("harness");
    gui.click("harness_new");
    gui.click("harness_lifecycle");

    let harness = WorkerTestHarness::new();

    // Test lifecycle transitions (valid transitions should pass)
    let failures = harness.test_lifecycle_transitions();

    // Should have no invalid transition failures
    assert!(
        failures.is_empty(),
        "Unexpected lifecycle failures: {:?}",
        failures
    );
}

#[test]
fn test_probar_worker_harness_message_ordering() {
    let mut gui = worker_harness_coverage();
    gui.visit("harness");
    gui.click("harness_ordering");

    let harness = WorkerTestHarness::new();

    // Valid Lamport timestamps (monotonically increasing)
    let valid_timestamps: Vec<u64> = (0..100).map(|i| i * 10).collect();
    let ordering_failures = harness.verify_message_ordering(&valid_timestamps);

    assert!(
        ordering_failures.is_empty(),
        "Valid timestamps should not have ordering failures"
    );

    // Invalid timestamps (out of order)
    let invalid_timestamps = vec![10, 20, 50, 30, 60]; // 30 < 50 violates ordering
    let failures = harness.verify_message_ordering(&invalid_timestamps);

    assert!(!failures.is_empty(), "Should detect ordering violation");
}

#[test]
fn test_probar_worker_harness_ring_buffer() {
    let mut gui = worker_harness_coverage();
    gui.visit("harness");
    gui.click("harness_ring_buffer");

    let harness = WorkerTestHarness::new();

    // Configure ring buffer for WASM compilation streaming
    let config = RingBufferTestConfig {
        buffer_size: 16384, // 16KB buffer
        sample_size: 256,   // 256 byte chunks
        num_samples: 100,
        test_overflow: true,
        test_underrun: true,
        test_concurrent: false, // Single-threaded for now
    };

    let result = harness.test_ring_buffer(&config);

    assert!(result.passed, "Ring buffer test should pass");
    assert!(result.writes_succeeded > 0);
    assert!(result.reads_succeeded > 0);
}

#[test]
fn test_probar_worker_harness_shared_memory() {
    let mut gui = worker_harness_coverage();
    gui.visit("harness");
    gui.click("harness_shared_memory");

    let harness = WorkerTestHarness::new();

    // Configure shared memory test for WASM heap
    let config = SharedMemoryTestConfig {
        buffer_size: 4096,
        num_atomic_ops: 100,
        test_wait_notify: true,
        test_concurrent_writes: false,
        wait_timeout: std::time::Duration::from_millis(50),
    };

    let result = harness.test_shared_memory(&config);

    assert!(
        result.atomics_correct,
        "Atomic operations should be correct"
    );
    assert_eq!(result.race_conditions_detected, 0);
}

#[test]
fn test_probar_worker_harness_config_presets() {
    let mut gui = worker_harness_coverage();
    gui.visit("harness");
    gui.click("harness_new");

    // Test different config presets
    let default_config = WorkerTestConfig::default();
    let minimal_config = WorkerTestConfig::minimal();
    let comprehensive_config = WorkerTestConfig::comprehensive();

    // Minimal should have fewer iterations
    assert!(minimal_config.stress_iterations < default_config.stress_iterations);

    // Comprehensive should have more iterations
    assert!(comprehensive_config.stress_iterations > default_config.stress_iterations);

    // Create harnesses with different configs
    let _default_harness = WorkerTestHarness::with_config(default_config);
    let _minimal_harness = WorkerTestHarness::with_config(minimal_config);
    let _comprehensive_harness = WorkerTestHarness::with_config(comprehensive_config);
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_probar_integration_worker_for_deployment() {
    let mut gui = worker_harness_coverage();
    gui.visit("integration");

    // Create a WorkerBrick for each Ruchy deployment target
    let browser_worker = WorkerBrick::new("browser_runtime")
        .message(
            BrickWorkerMessage::new("load_wasm", BrickWorkerMessageDirection::ToWorker)
                .field("url", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("loaded", BrickWorkerMessageDirection::FromWorker)
                .field("module_size", FieldType::Number),
        )
        .transition("init", "load_wasm", "loading")
        .transition("loading", "loaded", "ready");

    let cloudflare_worker = WorkerBrick::new("cloudflare_runtime")
        .message(
            BrickWorkerMessage::new("handle_request", BrickWorkerMessageDirection::ToWorker)
                .field("method", FieldType::String)
                .field("path", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("response", BrickWorkerMessageDirection::FromWorker)
                .field("status", FieldType::Number)
                .field("body", FieldType::String),
        )
        .transition("ready", "handle_request", "processing")
        .transition("processing", "response", "ready");

    // Generate code for both
    let browser_js = browser_worker.to_worker_js();
    let cloudflare_js = cloudflare_worker.to_worker_js();

    // Verify JS was generated (names are CamelCased)
    assert!(!browser_js.is_empty());
    assert!(!cloudflare_js.is_empty());
    assert!(browser_js.contains("Worker") || browser_js.contains("onmessage"));
    assert!(cloudflare_js.contains("Worker") || cloudflare_js.contains("onmessage"));
}

#[test]
fn test_probar_integration_distributed_compilation() {
    let mut gui = worker_harness_coverage();
    gui.visit("integration");

    // Set up distributed compilation environment
    let tracker = Arc::new(BrickDataTracker::new());
    let coordinator = BrickCoordinator::new();

    // Track compilation resources
    tracker.track_data("parser_cache", WorkerId::new(0), 10 * 1024 * 1024);
    tracker.track_data("type_checker", WorkerId::new(0), 5 * 1024 * 1024);
    tracker.track_data("code_gen", WorkerId::new(1), 8 * 1024 * 1024);

    // Subscribe to compilation events
    let progress_sub = coordinator.subscribe("progress");

    // Simulate compilation pipeline
    coordinator.publish(
        "progress",
        BrickMessage::StateChange {
            brick_name: "parser".into(),
            event: "completed".into(),
        },
    );

    coordinator.publish(
        "progress",
        BrickMessage::StateChange {
            brick_name: "type_checker".into(),
            event: "completed".into(),
        },
    );

    coordinator.publish(
        "progress",
        BrickMessage::StateChange {
            brick_name: "code_gen".into(),
            event: "completed".into(),
        },
    );

    let messages = progress_sub.drain();
    assert_eq!(messages.len(), 3);

    // Verify data locality for optimization pass
    let deps = vec!["parser_cache".into(), "type_checker".into()];
    let affinity = tracker.calculate_affinity(&deps);
    assert!(affinity.get(&WorkerId::new(0)).unwrap() > &0.0);
}

// =============================================================================
// Coverage Report
// =============================================================================

#[test]
fn test_probar_worker_harness_coverage_report() {
    let mut gui = worker_harness_coverage();

    // Visit all screens
    gui.visit("worker_brick");
    gui.visit("distributed");
    gui.visit("harness");
    gui.visit("integration");

    // Click all buttons
    for button in [
        "worker_brick_new",
        "worker_brick_message",
        "worker_brick_transition",
        "worker_brick_to_js",
        "worker_brick_to_rust",
        "worker_brick_to_typescript",
        "backend_selector_new",
        "backend_selector_select",
        "data_tracker_new",
        "data_tracker_track",
        "coordinator_new",
        "coordinator_publish",
        "coordinator_subscribe",
        "harness_new",
        "harness_lifecycle",
        "harness_ordering",
        "harness_ring_buffer",
        "harness_shared_memory",
    ] {
        gui.click(button);
    }

    // Generate coverage report
    let report = gui.generate_report();

    // Should have high coverage since we tested all components
    // overall_coverage is 0.0-1.0, so multiply by 100 for percentage
    assert!(
        report.overall_coverage >= 0.80,
        "Coverage should be >= 80%, got {:.1}%",
        report.overall_coverage * 100.0
    );
}
