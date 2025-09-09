use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ruchy_notebook::{memory, vm, converter, error};
use serde_json;
use std::collections::HashMap;
use std::time::Duration;

// Benchmark VM execution performance
fn benchmark_vm_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_execution");
    group.measurement_time(Duration::from_secs(10));
    
    let test_cases = vec![
        ("simple_arithmetic", "1 + 2 * 3"),
        ("variable_assignment", "let x = 42; x * 2"),
        ("string_operations", r#""hello" + " " + "world""#),
        ("complex_expression", "let a = 10; let b = 20; (a + b) * (a - b)"),
        ("function_call", "fun add(x, y) { x + y }; add(10, 20)"),
    ];
    
    for (name, code) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("bytecode_vm", name),
            &code,
            |b, code| {
                b.iter(|| {
                    let mut vm = vm::VirtualMachine::new();
                    let mut compiler = vm::Compiler::new();
                    
                    match compiler.compile_expression(black_box(code)) {
                        Ok(module) => {
                            let _ = vm.execute(&module);
                        }
                        Err(_) => {}
                    }
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark memory allocation patterns
fn benchmark_memory_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_management");
    group.measurement_time(Duration::from_secs(5));
    
    // Arena allocator benchmarks
    group.bench_function("arena_allocate_small", |b| {
        b.iter(|| {
            let arena = memory::Arena::new();
            for i in 0..1000 {
                let _ = arena.alloc(black_box(i as u32));
            }
        })
    });
    
    group.bench_function("arena_allocate_large", |b| {
        b.iter(|| {
            let arena = memory::Arena::new();
            for i in 0..100 {
                let data: Vec<u8> = (0..1024).map(|_| black_box(i as u8)).collect();
                let _ = arena.alloc(data);
            }
        })
    });
    
    // Slab allocator benchmarks
    group.bench_function("slab_allocate_handles", |b| {
        b.iter(|| {
            let mut slab = memory::SlabAllocator::new();
            let mut handles = Vec::new();
            
            for i in 0..1000 {
                let handle = slab.alloc(black_box(format!("value_{}", i)));
                handles.push(handle);
            }
            
            // Access some handles to prevent optimization
            for handle in handles.iter().take(10) {
                let _ = slab.get(&handle);
            }
        })
    });
    
    group.finish();
}

// Simple notebook structure for benchmarks
#[derive(serde::Serialize, serde::Deserialize)]
struct NotebookCell {
    cell_type: String,
    source: Vec<String>,
    outputs: Vec<serde_json::Value>,
    execution_count: Option<usize>,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NotebookDocument {
    cells: Vec<NotebookCell>,
    metadata: HashMap<String, serde_json::Value>,
}

impl NotebookDocument {
    fn new() -> Self {
        Self {
            cells: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    fn add_cell(&mut self, cell: NotebookCell) {
        self.cells.push(cell);
    }
}

// Benchmark notebook serialization/deserialization
fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    group.measurement_time(Duration::from_secs(5));
    
    let mut notebook = NotebookDocument::new();
    
    // Add various cell types
    for i in 0..100 {
        notebook.add_cell(NotebookCell {
            cell_type: if i % 2 == 0 { "code".to_string() } else { "markdown".to_string() },
            source: vec![format!("Cell content {}", i)],
            outputs: if i % 3 == 0 { 
                vec![serde_json::json!({"output_type": "stream", "text": format!("Output {}", i)})]
            } else { 
                vec![] 
            },
            execution_count: Some(i),
            metadata: HashMap::new(),
        });
    }
    
    group.bench_function("serialize_notebook", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(black_box(&notebook));
        })
    });
    
    let serialized = serde_json::to_string(&notebook).unwrap();
    
    group.bench_function("deserialize_notebook", |b| {
        b.iter(|| {
            let _: NotebookDocument = serde_json::from_str(black_box(&serialized)).unwrap();
        })
    });
    
    group.finish();
}

// Benchmark demo conversion performance
fn benchmark_demo_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("demo_conversion");
    group.measurement_time(Duration::from_secs(5));
    
    let demo_contents = vec![
        (
            "simple_demo",
            r#"// Basic math example
println("Hello, Ruchy!");
let x = 42;
x * 2"#
        ),
        (
            "complex_demo", 
            r#"// Complex demo with multiple sections
// ## Data Processing
let data = [1, 2, 3, 4, 5];
let sum = data.reduce(fun(acc, x) { acc + x }, 0);

// ## String Operations  
let message = f"Sum is {sum}";
println(message);

// ## Control Flow
if sum > 10 {
    println("Large sum!");
} else {
    println("Small sum");
}

// ## Functions
fun fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fibonacci(10)"#
        ),
    ];
    
    for (name, content) in demo_contents {
        group.bench_with_input(
            BenchmarkId::new("parse_demo", name),
            &content,
            |b, content| {
                b.iter(|| {
                    let parser = converter::DemoParser::new();
                    let _ = parser.parse(black_box(content));
                })
            },
        );
        
        let parser = converter::DemoParser::new();
        let cells = parser.parse(content).unwrap_or_default();
        
        group.bench_with_input(
            BenchmarkId::new("convert_to_notebook", name),
            &cells,
            |b, cells| {
                b.iter(|| {
                    let converter = converter::DemoConverter::new();
                    let _ = converter.to_notebook(black_box(cells));
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark error handling and suggestions
fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    group.measurement_time(Duration::from_secs(5));
    
    let suggestion_engine = error::SuggestionEngine::new();
    let test_variables = vec!["length", "width", "height", "data", "result", "value"];
    
    for var in test_variables {
        suggestion_engine.add_variable(var.to_string());
    }
    
    let typos = vec![
        "lenght", "widht", "heigh", "dat", "resul", "valu"
    ];
    
    for typo in typos {
        group.bench_with_input(
            BenchmarkId::new("levenshtein_suggestions", typo),
            &typo,
            |b, typo| {
                b.iter(|| {
                    let _ = suggestion_engine.suggest_for_undefined(black_box(typo));
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark WASM-specific operations
#[cfg(feature = "wasm")]
fn benchmark_wasm_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_operations");
    group.measurement_time(Duration::from_secs(5));
    
    group.bench_function("wasm_notebook_creation", |b| {
        b.iter(|| {
            let _ = wasm::WasmNotebook::new();
        })
    });
    
    let mut notebook = wasm::WasmNotebook::new();
    let test_code = "let x = 42; x * 2";
    
    group.bench_function("wasm_code_execution", |b| {
        b.iter(|| {
            let _ = notebook.execute(black_box(test_code));
        })
    });
    
    group.finish();
}

// Performance regression tests
fn benchmark_performance_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_targets");
    group.measurement_time(Duration::from_secs(10));
    
    // Target: <50ms for cell execution
    group.bench_function("cell_execution_target", |b| {
        b.iter(|| {
            let mut vm = vm::VirtualMachine::new();
            let mut compiler = vm::Compiler::new();
            
            let code = "let data = [1, 2, 3, 4, 5]; data.map(fun(x) { x * x }).reduce(fun(a, b) { a + b }, 0)";
            
            match compiler.compile_expression(black_box(code)) {
                Ok(module) => {
                    let _ = vm.execute(&module);
                }
                Err(_) => {}
            }
        })
    });
    
    // Target: <200ms for notebook loading (100 cells)
    group.bench_function("notebook_loading_target", |b| {
        b.iter(|| {
            let mut notebook = NotebookDocument::new();
            
            for i in 0..100 {
                notebook.add_cell(NotebookCell {
                    cell_type: "code".to_string(),
                    source: vec![format!("println(\"Cell {}\")", i)],
                    outputs: vec![],
                    execution_count: Some(i),
                    metadata: HashMap::new(),
                });
            }
            
            let _ = serde_json::to_string(&notebook);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_vm_execution,
    benchmark_memory_management,
    benchmark_serialization,
    benchmark_demo_conversion,
    benchmark_error_handling,
    benchmark_performance_targets
);

#[cfg(feature = "wasm")]
criterion_group!(
    wasm_benches,
    benchmark_wasm_operations
);

#[cfg(not(feature = "wasm"))]
criterion_main!(benches);

#[cfg(feature = "wasm")]
criterion_main!(benches, wasm_benches);