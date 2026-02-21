//! Probar Distributed Worker Demo
//!
//! Demonstrates distributed execution capabilities for Ruchy's
//! WASM compilation pipeline using jugar-probar.
//!
//! Run with: cargo run --example probar_distributed

use jugar_probar::brick::distributed::{
    BackendSelector, BrickCoordinator, BrickDataTracker, BrickMessage, WorkerId,
};
use std::sync::Arc;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Probar Distributed Worker Demo for Ruchy                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    demo_backend_selection();
    demo_data_locality();
    demo_pub_sub_coordination();
    demo_compilation_pipeline();
}

/// Demonstrate backend selection for Ruchy compilation workloads
fn demo_backend_selection() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  1. Backend Selection for Compilation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Configure backend selection for Ruchy's compilation workloads
    let selector = BackendSelector::new()
        .with_gpu_threshold(500_000) // Use GPU for large source files
        .with_simd_threshold(10_000) // SIMD for medium files
        .with_cpu_max_threshold(50_000_000); // CPU max threshold

    println!("  BackendSelector Configuration:");
    println!("    ├─ GPU threshold: >= 500,000 chars");
    println!("    ├─ SIMD threshold: >= 10,000 chars");
    println!("    └─ CPU max: 50,000,000 chars\n");

    // Test different source file sizes
    let test_cases = [
        (500, "Tiny script (500 chars)"),
        (5_000, "Small module (5KB)"),
        (25_000, "Medium module (25KB)"),
        (100_000, "Large module (100KB)"),
        (750_000, "Very large module (750KB)"),
    ];

    println!("  Selection Results:");
    println!("  ┌────────────────────────────┬──────────────┬──────────┐");
    println!("  │ Source File                │ Size (chars) │ Backend  │");
    println!("  ├────────────────────────────┼──────────────┼──────────┤");

    for (size, desc) in &test_cases {
        let gpu_available = *size > 100_000; // Simulate GPU availability for large files
        let backend = selector.select(*size, gpu_available);
        println!("  │ {:<26} │ {:>12} │ {:?}    │", desc, size, backend);
    }
    println!("  └────────────────────────────┴──────────────┴──────────┘");

    println!("\n  Backend Capabilities:");
    println!("    ├─ CPU: Always available, good for small files");
    println!("    ├─ SIMD: Vectorized parsing, 2-4x faster for medium files");
    println!("    ├─ GPU: Parallel compilation, best for large codebases");
    println!("    └─ Remote: Distributed compilation (future)");
    println!();
}

/// Demonstrate data locality tracking for compilation resources
fn demo_data_locality() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  2. Data Locality for Compilation Resources");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let tracker = BrickDataTracker::new();

    // Simulate compilation workers with different cached data
    let worker0 = WorkerId::new(0);
    let worker1 = WorkerId::new(1);
    let worker2 = WorkerId::new(2);

    println!("  Registering compilation resources:");
    println!("    Worker 0: parser_cache (2MB), type_checker (3MB), stdlib (5MB)");
    println!("    Worker 1: parser_cache (2MB), optimizer (4MB)");
    println!("    Worker 2: code_gen (6MB), stdlib (5MB)");
    println!();

    // Track data locations
    tracker.track_data("parser_cache", worker0, 2 * 1024 * 1024);
    tracker.track_data("type_checker", worker0, 3 * 1024 * 1024);
    tracker.track_data("stdlib", worker0, 5 * 1024 * 1024);

    tracker.track_data("parser_cache", worker1, 2 * 1024 * 1024);
    tracker.track_data("optimizer", worker1, 4 * 1024 * 1024);

    tracker.track_data("code_gen", worker2, 6 * 1024 * 1024);
    tracker.track_data("stdlib", worker2, 5 * 1024 * 1024);

    // Query data locations
    println!("  Data Location Queries:");
    for key in &[
        "parser_cache",
        "type_checker",
        "stdlib",
        "optimizer",
        "code_gen",
    ] {
        let workers = tracker.get_workers_for_data(key);
        let worker_ids: Vec<_> = workers.iter().map(|w| w.0).collect();
        println!("    {} → workers {:?}", key, worker_ids);
    }
    println!();

    // Calculate affinity for different compilation tasks
    println!("  Task Affinity Scores:");

    // Full compilation needs parser + type_checker + code_gen
    let full_compile_deps = vec![
        "parser_cache".into(),
        "type_checker".into(),
        "code_gen".into(),
    ];
    let full_affinity = tracker.calculate_affinity(&full_compile_deps);
    println!("\n    Full Compilation (parser + type_checker + code_gen):");
    print_affinity_scores(&full_affinity);

    // Type checking needs parser + type_checker + stdlib
    let type_check_deps = vec![
        "parser_cache".into(),
        "type_checker".into(),
        "stdlib".into(),
    ];
    let type_affinity = tracker.calculate_affinity(&type_check_deps);
    println!("\n    Type Checking (parser + type_checker + stdlib):");
    print_affinity_scores(&type_affinity);

    // Optimization needs parser + optimizer
    let optimize_deps = vec!["parser_cache".into(), "optimizer".into()];
    let opt_affinity = tracker.calculate_affinity(&optimize_deps);
    println!("\n    Optimization (parser + optimizer):");
    print_affinity_scores(&opt_affinity);
    println!();
}

/// Demonstrate PUB/SUB coordination for compilation events
fn demo_pub_sub_coordination() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  3. PUB/SUB Coordination");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let coordinator = BrickCoordinator::new();

    // Create subscriptions for different event types
    let compile_sub = coordinator.subscribe("compilation");
    let error_sub = coordinator.subscribe("errors");
    let progress_sub = coordinator.subscribe("progress");

    println!("  Subscriptions:");
    println!("    ├─ Topic: {} (compilation events)", compile_sub.topic());
    println!("    ├─ Topic: {} (error events)", error_sub.topic());
    println!("    └─ Topic: {} (progress updates)", progress_sub.topic());
    println!();

    // Simulate compilation pipeline events
    println!("  Publishing compilation events...");

    coordinator.publish(
        "progress",
        BrickMessage::StateChange {
            brick_name: "parser".into(),
            event: "started".into(),
        },
    );

    coordinator.publish(
        "progress",
        BrickMessage::StateChange {
            brick_name: "parser".into(),
            event: "completed".into(),
        },
    );

    coordinator.publish(
        "compilation",
        BrickMessage::StateChange {
            brick_name: "type_checker".into(),
            event: "started".into(),
        },
    );

    coordinator.publish(
        "errors",
        BrickMessage::StateChange {
            brick_name: "type_checker".into(),
            event: "warning: unused variable 'x'".into(),
        },
    );

    coordinator.publish(
        "compilation",
        BrickMessage::StateChange {
            brick_name: "type_checker".into(),
            event: "completed".into(),
        },
    );

    coordinator.publish(
        "compilation",
        BrickMessage::StateChange {
            brick_name: "code_gen".into(),
            event: "completed".into(),
        },
    );

    // Drain and display messages
    let compile_msgs = compile_sub.drain();
    let error_msgs = error_sub.drain();
    let progress_msgs = progress_sub.drain();

    println!("\n  Received Messages:");
    println!("    compilation topic: {} messages", compile_msgs.len());
    for msg in &compile_msgs {
        if let BrickMessage::StateChange { brick_name, event } = msg {
            println!("      • {} → {}", brick_name, event);
        }
    }

    println!("    errors topic: {} messages", error_msgs.len());
    for msg in &error_msgs {
        if let BrickMessage::StateChange { brick_name, event } = msg {
            println!("      • {} → {}", brick_name, event);
        }
    }

    println!("    progress topic: {} messages", progress_msgs.len());
    for msg in &progress_msgs {
        if let BrickMessage::StateChange { brick_name, event } = msg {
            println!("      • {} → {}", brick_name, event);
        }
    }
    println!();
}

/// Demonstrate a complete compilation pipeline
fn demo_compilation_pipeline() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  4. Ruchy Compilation Pipeline");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let tracker = Arc::new(BrickDataTracker::new());
    let coordinator = BrickCoordinator::new();
    let selector = BackendSelector::new()
        .with_simd_threshold(5_000)
        .with_gpu_threshold(100_000);

    // Set up workers
    tracker.track_data("lexer_tables", WorkerId::new(0), 1024 * 1024);
    tracker.track_data("grammar_cache", WorkerId::new(0), 2 * 1024 * 1024);
    tracker.track_data("type_env", WorkerId::new(1), 3 * 1024 * 1024);
    tracker.track_data("wasm_templates", WorkerId::new(2), 4 * 1024 * 1024);

    // Subscribe to pipeline events
    let pipeline_sub = coordinator.subscribe("pipeline");

    println!("  Pipeline Stages:");
    println!("    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐");
    println!("    │   Lexer     │ -> │   Parser    │ -> │ Type Check  │");
    println!("    │  (Worker 0) │    │  (Worker 0) │    │  (Worker 1) │");
    println!("    └─────────────┘    └─────────────┘    └─────────────┘");
    println!("                                                │");
    println!("    ┌─────────────┐    ┌─────────────┐         │");
    println!("    │  Optimize   │ <- │  Code Gen   │ <───────┘");
    println!("    │  (Worker 1) │    │  (Worker 2) │");
    println!("    └─────────────┘    └─────────────┘");
    println!();

    // Simulate compilation of different file sizes
    let source_sizes = [
        (1_000, "small.ruchy"),
        (20_000, "medium.ruchy"),
        (150_000, "large.ruchy"),
    ];

    println!("  Compilation Runs:");
    for (size, filename) in &source_sizes {
        let backend = selector.select(*size, true);
        let deps = vec!["lexer_tables".into(), "grammar_cache".into()];
        let affinity = tracker.calculate_affinity(&deps);
        let best_worker = affinity
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(w, _)| w.0)
            .unwrap_or(0);

        coordinator.publish(
            "pipeline",
            BrickMessage::StateChange {
                brick_name: (*filename).into(),
                event: format!("compiled on {:?} (worker {})", backend, best_worker),
            },
        );

        println!(
            "    {} ({} chars): {:?} backend, worker {}",
            filename, size, backend, best_worker
        );
    }

    let msgs = pipeline_sub.drain();
    println!("\n  Pipeline Events: {} total", msgs.len());

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Demo complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// Helper to print affinity scores
fn print_affinity_scores(affinity: &std::collections::HashMap<WorkerId, f64>) {
    let mut sorted: Vec<_> = affinity.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    for (worker, score) in &sorted {
        let bar_len = (*score * 20.0) as usize;
        let bar: String = "█".repeat(bar_len);
        println!("      Worker {}: {:.2} {}", worker.0, score, bar);
    }
    if let Some((best, _)) = sorted.first() {
        println!("      → Best worker: {}", best.0);
    }
}
