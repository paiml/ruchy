//! Bytecode VM Performance Benchmarks
//!
//! OPT-021: Bytecode VM Performance Validation
//!
//! Validates the 98-99% speedup claims by comparing AST interpreter
//! vs bytecode VM execution across all Phase 1 & 2 features.
//!
//! Benchmarks organized by OPT tickets:
//! - OPT-001 to OPT-010: Phase 1 (Basic operations)
//! - OPT-011 to OPT-020: Phase 2 (Complex features)
//!
//! Run with: cargo bench --bench `bytecode_vm_performance`

#![allow(deprecated)] // black_box deprecated - will fix in separate ticket

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::bytecode::{Compiler, VM};

/// Helper to benchmark both AST and Bytecode VM execution
fn bench_both_modes(c: &mut Criterion, name: &str, code: &str) {
    let mut group = c.benchmark_group(name);

    // Parse once (shared by both modes)
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    // Benchmark AST interpreter
    group.bench_function("AST", |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            black_box(interpreter.eval_expr(&ast).expect("AST eval failed"))
        });
    });

    // Benchmark Bytecode VM
    group.bench_function("Bytecode", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new("bench".to_string());
            compiler.compile_expr(&ast).expect("Compile failed");
            let chunk = compiler.finalize();
            let mut vm = VM::new();
            black_box(vm.execute(&chunk).expect("VM run failed"))
        });
    });

    group.finish();
}

/// OPT-001 through OPT-007: Basic arithmetic and operations
fn bench_basic_operations(c: &mut Criterion) {
    bench_both_modes(c, "arithmetic_simple", "2 + 2");
    bench_both_modes(c, "arithmetic_complex", "((10 + 5) * 2 - 3) / 4");
    bench_both_modes(c, "unary_operations", "-5 + (-10) * 2");

    bench_both_modes(c, "variables_simple", "{ let x = 10; x }");
    bench_both_modes(c, "variables_multiple", "{ let x = 10; let y = 20; x + y }");

    bench_both_modes(c, "comparison", "{ let x = 10; let y = 20; x < y }");
    bench_both_modes(c, "logical_ops", "{ let a = true; let b = false; a && !b }");
}

/// OPT-006 to OPT-009: Loops and assignments
fn bench_loops_assignments(c: &mut Criterion) {
    // While loop with counter
    bench_both_modes(
        c,
        "while_loop_simple",
        r"{
            let mut sum = 0;
            let mut i = 0;
            while i < 10 {
                sum = sum + i;
                i = i + 1;
            }
            sum
        }"
    );

    // Assignment operations
    bench_both_modes(
        c,
        "assignments",
        r"{
            let mut x = 0;
            x = 10;
            x = x + 5;
            x = x * 2;
            x
        }"
    );
}

/// OPT-012: For loops
fn bench_for_loops(c: &mut Criterion) {
    bench_both_modes(
        c,
        "for_loop_simple",
        r"{
            let mut sum = 0;
            for i in [1, 2, 3, 4, 5] {
                sum = sum + i;
            }
            sum
        }"
    );

    bench_both_modes(
        c,
        "for_loop_nested",
        r"{
            let mut sum = 0;
            for i in [1, 2, 3] {
                for j in [1, 2] {
                    sum = sum + i * j;
                }
            }
            sum
        }"
    );
}

/// OPT-013: Array indexing
fn bench_arrays(c: &mut Criterion) {
    bench_both_modes(
        c,
        "array_indexing",
        r"{
            let arr = [10, 20, 30, 40, 50];
            arr[0] + arr[2] + arr[4]
        }"
    );

    bench_both_modes(
        c,
        "array_iteration",
        r"{
            let mut sum = 0;
            for x in [5, 10, 15, 20] {
                sum = sum + x;
            }
            sum
        }"
    );
}

/// OPT-014, OPT-015: Method calls and field access
fn bench_method_calls(c: &mut Criterion) {
    bench_both_modes(
        c,
        "string_methods",
        r#"{
            let s = "hello";
            s.len()
        }"#
    );

    bench_both_modes(
        c,
        "object_field_access",
        r"{
            let obj = { x: 10, y: 20 };
            obj.x + obj.y
        }"
    );
}

/// OPT-016, OPT-017: Object and tuple literals
fn bench_literals(c: &mut Criterion) {
    bench_both_modes(
        c,
        "object_literal",
        r#"{ name: "Alice", age: 30, score: 95 }"#
    );

    bench_both_modes(
        c,
        "tuple_literal",
        r"(1, 2, 3, 4, 5)"
    );
}

/// OPT-018: Match expressions
fn bench_match(c: &mut Criterion) {
    bench_both_modes(
        c,
        "match_simple",
        r"{
            let x = 2;
            match x {
                1 => 10,
                2 => 20,
                _ => 0,
            }
        }"
    );

    bench_both_modes(
        c,
        "match_complex",
        r"{
            let value = Some(42);
            match value {
                Some(x) => x * 2,
                None => 0,
            }
        }"
    );
}

/// OPT-019: Closures
fn bench_closures(c: &mut Criterion) {
    bench_both_modes(
        c,
        "closure_simple",
        r"{
            let x = 10;
            let f = |y| x + y;
            f(5)
        }"
    );

    bench_both_modes(
        c,
        "closure_capture",
        r"{
            let a = 5;
            let b = 10;
            let add = |x| a + b + x;
            add(3) + add(7)
        }"
    );
}

/// OPT-020: Non-literal collections
fn bench_non_literal_collections(c: &mut Criterion) {
    bench_both_modes(
        c,
        "array_with_variables",
        r"{
            let x = 10;
            let y = 20;
            let arr = [x, y, x + y];
            arr[0] + arr[1] + arr[2]
        }"
    );

    bench_both_modes(
        c,
        "tuple_with_expressions",
        r"{
            let a = 5;
            let b = 10;
            (a, b, a + b, a * b)
        }"
    );

    bench_both_modes(
        c,
        "object_with_variables",
        r#"{
            let name = "Bob";
            let age = 25;
            { name: name, age: age }
        }"#
    );
}

/// Comprehensive benchmark combining all features
fn bench_comprehensive(c: &mut Criterion) {
    bench_both_modes(
        c,
        "fibonacci_iterative",
        r"{
            let mut a = 0;
            let mut b = 1;
            let mut i = 0;
            while i < 10 {
                let temp = a + b;
                a = b;
                b = temp;
                i = i + 1;
            }
            b
        }"
    );

    bench_both_modes(
        c,
        "data_processing",
        r"{
            let data = [10, 20, 30, 40, 50];
            let mut sum = 0;
            let mut count = 0;

            for x in data {
                if x > 15 {
                    sum = sum + x;
                    count = count + 1;
                }
            }

            { sum: sum, count: count, avg: sum / count }
        }"
    );
}

criterion_group!(
    benches,
    bench_basic_operations,
    bench_loops_assignments,
    bench_for_loops,
    bench_arrays,
    bench_method_calls,
    bench_literals,
    bench_match,
    bench_closures,
    bench_non_literal_collections,
    bench_comprehensive
);

criterion_main!(benches);
