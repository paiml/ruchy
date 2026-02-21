//! Probar WorkerBrick Code Generation Demo
//!
//! Demonstrates automatic generation of Web Worker code for Ruchy's
//! WASM compilation and deployment targets using jugar-probar.
//!
//! Run with: cargo run --example probar_worker_brick

use jugar_probar::brick::worker::{
    BrickWorkerMessage, BrickWorkerMessageDirection, FieldType, WorkerBrick,
};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Probar WorkerBrick Demo for Ruchy WASM                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    demo_compiler_worker();
    demo_repl_worker();
    demo_deployment_workers();
    demo_field_types();
}

/// Demonstrate a worker for Ruchy compilation
fn demo_compiler_worker() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  1. Ruchy Compiler Worker");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let compiler_worker = WorkerBrick::new("ruchy_compiler")
        // Messages TO the worker
        .message(
            BrickWorkerMessage::new("compile", BrickWorkerMessageDirection::ToWorker)
                .field("source", FieldType::String)
                .field("filename", FieldType::String)
                .optional_field("optimize", FieldType::Boolean),
        )
        .message(
            BrickWorkerMessage::new("parse_only", BrickWorkerMessageDirection::ToWorker)
                .field("source", FieldType::String),
        )
        .message(BrickWorkerMessage::new(
            "cancel",
            BrickWorkerMessageDirection::ToWorker,
        ))
        // Messages FROM the worker
        .message(
            BrickWorkerMessage::new("compiled", BrickWorkerMessageDirection::FromWorker)
                .field("wasm", FieldType::SharedArrayBuffer)
                .field("size", FieldType::Number)
                .field("warnings", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("ast", BrickWorkerMessageDirection::FromWorker)
                .field("json", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("error", BrickWorkerMessageDirection::FromWorker)
                .field("message", FieldType::String)
                .field("line", FieldType::Number)
                .field("column", FieldType::Number),
        )
        .message(
            BrickWorkerMessage::new("progress", BrickWorkerMessageDirection::FromWorker)
                .field("stage", FieldType::String)
                .field("percent", FieldType::Number),
        )
        // State machine
        .transition("idle", "compile", "compiling")
        .transition("idle", "parse_only", "parsing")
        .transition("compiling", "compiled", "idle")
        .transition("compiling", "error", "idle")
        .transition("compiling", "cancel", "idle")
        .transition("parsing", "ast", "idle")
        .transition("parsing", "error", "idle");

    println!("  Worker Definition:");
    println!("    ├─ Name: ruchy_compiler");
    println!(
        "    ├─ ToWorker messages: {}",
        compiler_worker.to_worker_messages().len()
    );
    println!(
        "    └─ FromWorker messages: {}",
        compiler_worker.from_worker_messages().len()
    );

    println!("\n  State Machine:");
    println!("    idle ─compile─► compiling ─compiled─► idle");
    println!("         ─parse───► parsing ───ast─────► idle");
    println!("                    │");
    println!("                    └─error/cancel─► idle");

    // Generate JavaScript
    let js_code = compiler_worker.to_worker_js();
    println!("\n  Generated JavaScript ({} chars):", js_code.len());
    print_code_preview(&js_code, 10);
    println!();
}

/// Demonstrate a worker for Ruchy REPL
fn demo_repl_worker() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  2. Ruchy REPL Worker");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let repl_worker = WorkerBrick::new("ruchy_repl")
        .message(
            BrickWorkerMessage::new("eval", BrickWorkerMessageDirection::ToWorker)
                .field("code", FieldType::String)
                .optional_field("timeout_ms", FieldType::Number),
        )
        .message(BrickWorkerMessage::new(
            "reset",
            BrickWorkerMessageDirection::ToWorker,
        ))
        .message(
            BrickWorkerMessage::new("result", BrickWorkerMessageDirection::FromWorker)
                .field("output", FieldType::String)
                .field("value_type", FieldType::String)
                .field("elapsed_ms", FieldType::Number),
        )
        .message(
            BrickWorkerMessage::new("error", BrickWorkerMessageDirection::FromWorker)
                .field("message", FieldType::String),
        )
        .message(BrickWorkerMessage::new(
            "ready",
            BrickWorkerMessageDirection::FromWorker,
        ))
        .transition("init", "ready", "ready")
        .transition("ready", "eval", "evaluating")
        .transition("evaluating", "result", "ready")
        .transition("evaluating", "error", "ready")
        .transition("ready", "reset", "init");

    // Generate Rust bindings
    let rust_code = repl_worker.to_rust_bindings();
    println!("  Generated Rust Bindings ({} chars):", rust_code.len());
    print_code_preview(&rust_code, 15);
    println!();
}

/// Demonstrate workers for different deployment targets
fn demo_deployment_workers() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  3. Deployment Target Workers");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Browser runtime worker
    let browser_worker = WorkerBrick::new("browser_runtime")
        .message(
            BrickWorkerMessage::new("load_wasm", BrickWorkerMessageDirection::ToWorker)
                .field("url", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("call", BrickWorkerMessageDirection::ToWorker)
                .field("function", FieldType::String)
                .field("args", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("loaded", BrickWorkerMessageDirection::FromWorker)
                .field("exports", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("result", BrickWorkerMessageDirection::FromWorker)
                .field("value", FieldType::String),
        )
        .transition("init", "load_wasm", "loading")
        .transition("loading", "loaded", "ready")
        .transition("ready", "call", "executing")
        .transition("executing", "result", "ready");

    // Cloudflare Workers runtime
    let cloudflare_worker = WorkerBrick::new("cloudflare_runtime")
        .message(
            BrickWorkerMessage::new("handle_request", BrickWorkerMessageDirection::ToWorker)
                .field("method", FieldType::String)
                .field("path", FieldType::String)
                .field("body", FieldType::String),
        )
        .message(
            BrickWorkerMessage::new("response", BrickWorkerMessageDirection::FromWorker)
                .field("status", FieldType::Number)
                .field("headers", FieldType::String)
                .field("body", FieldType::String),
        )
        .transition("ready", "handle_request", "processing")
        .transition("processing", "response", "ready");

    println!("  Browser Runtime Worker:");
    let browser_ts = browser_worker.to_typescript_defs();
    println!("    TypeScript definitions ({} chars)", browser_ts.len());

    println!("\n  Cloudflare Workers Runtime:");
    let cloudflare_ts = cloudflare_worker.to_typescript_defs();
    println!("    TypeScript definitions ({} chars)", cloudflare_ts.len());

    println!("\n  TypeScript for Cloudflare:");
    print_code_preview(&cloudflare_ts, 20);
    println!();
}

/// Demonstrate field type mappings
fn demo_field_types() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  4. Field Type Mappings");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("  Rust → JavaScript → TypeScript mappings:\n");
    println!("  ┌─────────────────────────────────────────────────────────┐");
    println!("  │ FieldType             │ TypeScript      │ Rust          │");
    println!("  ├─────────────────────────────────────────────────────────┤");

    let types = [
        FieldType::String,
        FieldType::Number,
        FieldType::Boolean,
        FieldType::SharedArrayBuffer,
        FieldType::Float32Array,
        FieldType::Optional(Box::new(FieldType::Number)),
    ];

    for ft in &types {
        println!(
            "  │ {:<21} │ {:<15} │ {:<13} │",
            format!("{:?}", ft),
            ft.to_typescript(),
            ft.to_rust()
        );
    }
    println!("  └─────────────────────────────────────────────────────────┘");

    println!("\n  Zero-JS Benefits for Ruchy:");
    println!("    ├─ Compiler worker code generated from Rust specs");
    println!("    ├─ Type-safe message protocols for REPL");
    println!("    ├─ State machine prevents invalid transitions");
    println!("    ├─ Consistent API across browser/Cloudflare targets");
    println!("    └─ No hand-written JavaScript maintenance");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Demo complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// Helper to print code preview
fn print_code_preview(code: &str, lines: usize) {
    println!("  ┌─────────────────────────────────────────────────────────┐");
    for (i, line) in code.lines().take(lines).enumerate() {
        let truncated = if line.len() > 55 {
            format!("{}...", &line[..52])
        } else {
            line.to_string()
        };
        println!("  │ {:2}. {:<53} │", i + 1, truncated);
    }
    let total_lines = code.lines().count();
    if total_lines > lines {
        println!(
            "  │     ... ({} more lines)                               │",
            total_lines - lines
        );
    }
    println!("  └─────────────────────────────────────────────────────────┘");
}
