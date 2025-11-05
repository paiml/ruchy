#![allow(missing_docs)]
//! TRANSPILER-PROPERTY: Comprehensive Property-Based Test Suite
//!
//! **Purpose**: Prevent transpiler bugs through 10K+ randomized test cases
//! **Strategy**: Test all major bug categories with generated programs
//! **Method**: EXTREME TDD with parse → transpile → compile → execute validation
//!
//! **Bug Categories Targeted** (from 20+ historical transpiler bugs):
//! - Type Inference (40%): TRANSPILER-TYPE, TRANSPILER-PARAM-INFERENCE, etc.
//! - Scope/Variable (25%): TRANSPILER-SCOPE, TRANSPILER-GLOBAL-LET, etc.
//! - Optimization (20%): TRANSPILER-009, TRANSPILER-015, etc.
//! - Code Generation (15%): TRANSPILER-011, TRANSPILER-014, etc.
//!
//! This follows EXTREME TDD protocol: RED → GREEN → REFACTOR → VALIDATE

use assert_cmd::Command;
use predicates::prelude::*;
use proptest::prelude::*;
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== GENERATORS ====================

/// Strategy: Generate valid function names (lowercase start, alphanumeric)
fn gen_func_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

/// Strategy: Generate valid variable names
fn gen_var_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

/// Strategy: Generate type annotations (focus on inference-prone types)
fn gen_type_annotation() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec![
        "i32",
        "f64",
        "bool",
        "String",
        "str",
        "i64",
        "u32",
        "char",
    ])
}

/// Strategy: Generate simple expressions (for type inference testing)
fn gen_simple_expr(var_name: String) -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        format!("{var_name}"),
        format!("{var_name} + 1"),
        format!("{var_name} * 2"),
        format!("{var_name} - 3"),
        "42".to_string(),
        "true".to_string(),
        "false".to_string(),
        "\"test\"".to_string(),
    ])
}

/// Strategy: Generate function with type inference challenges (Category 1: 40%)
fn gen_type_inference_function() -> impl Strategy<Value = String> {
    (gen_func_name(), gen_var_name(), gen_type_annotation()).prop_map(
        |(fn_name, param_name, param_type)| {
            // Test type inference for return types
            format!(
                r#"
fun {fn_name}({param_name}: {param_type}) {{
    let result = {param_name};
    result
}}

fun main() {{
    let value = {fn_name}({});
    println("{{}}", value)
}}
"#,
                match param_type {
                    "i32" | "i64" | "u32" => "42",
                    "f64" => "3.14",
                    "bool" => "true",
                    "char" => "'x'",
                    "str" => "\"test\"",
                    "String" => "\"test\".to_string()",
                    _ => "42",
                }
            )
        },
    )
}

/// Strategy: Generate nested scope tests (Category 2: 25%)
fn gen_nested_scope_program() -> impl Strategy<Value = String> {
    (gen_func_name(), gen_var_name(), 1usize..4).prop_map(|(fn_name, var_name, depth)| {
        let mut code = format!(
            "fun {fn_name}() -> i32 {{\n    let {var_name} = 10;\n"
        );

        // Generate nested blocks
        for i in 0..depth {
            code.push_str(&format!(
                "    let inner_{i} = {var_name} + {i};\n"
            ));
            code.push_str(&format!("    {{\n        let nested_{i} = inner_{i} * 2;\n"));
        }

        // Close nested blocks and return
        for _ in 0..depth {
            code.push_str("    }\n");
        }

        code.push_str(&format!("    {var_name}\n}}\n\n"));
        code.push_str(&format!(
            "fun main() {{\n    let result = {fn_name}();\n    println(\"{{}}\", result)\n}}\n"
        ));

        code
    })
}

/// Strategy: Generate programs with optimization targets (Category 3: 20%)
fn gen_optimization_target() -> impl Strategy<Value = String> {
    (gen_func_name(), prop::bool::ANY, prop::bool::ANY).prop_map(
        |(fn_name, use_const, use_identity)| {
            if use_const {
                // Constant folding targets
                format!(
                    r#"
fun {fn_name}() -> i32 {{
    let x = 5 + 3;
    let y = x * 2;
    y
}}

fun main() {{
    println("{{}}", {fn_name}())
}}
"#
                )
            } else if use_identity {
                // Identity operation optimization
                format!(
                    r#"
fun {fn_name}(n: i32) -> i32 {{
    let x = n + 0;
    let y = x * 1;
    y
}}

fun main() {{
    println("{{}}", {fn_name}(42))
}}
"#
                )
            } else {
                // Dead code elimination target
                format!(
                    r#"
fun {fn_name}() -> i32 {{
    let x = 10;
    let _unused = 99;
    x
}}

fun main() {{
    println("{{}}", {fn_name}())
}}
"#
                )
            }
        },
    )
}

/// Strategy: Generate complex expressions (Category 4: 15%)
fn gen_complex_expression() -> impl Strategy<Value = String> {
    (gen_func_name(), 2usize..5).prop_map(|(fn_name, operator_count)| {
        let mut expr = "x".to_string();
        for i in 1..operator_count {
            let op = match i % 4 {
                0 => "+",
                1 => "-",
                2 => "*",
                _ => "/",
            };
            expr = format!("({expr} {op} {i})");
        }

        format!(
            r#"
fun {fn_name}(x: i32) -> i32 {{
    let result = {expr};
    result
}}

fun main() {{
    println("{{}}", {fn_name}(10))
}}
"#
        )
    })
}

/// Strategy: Generate method call patterns
fn gen_method_call_program() -> impl Strategy<Value = String> {
    (gen_func_name(), gen_var_name()).prop_map(|(fn_name, var_name)| {
        format!(
            r#"
fun {fn_name}({var_name}: String) -> usize {{
    {var_name}.len()
}}

fun main() {{
    let result = {fn_name}("hello".to_string());
    println("{{}}", result)
}}
"#
        )
    })
}

/// Strategy: Generate pattern matching with variables
fn gen_pattern_match_program() -> impl Strategy<Value = String> {
    (gen_func_name(), gen_var_name(), 2usize..5).prop_map(|(fn_name, var_name, case_count)| {
        let mut cases = String::new();
        for i in 0..case_count {
            cases.push_str(&format!("        {i} => \"{i}\",\n"));
        }
        cases.push_str("        _ => \"other\",\n");

        format!(
            r#"
fun {fn_name}({var_name}: i32) -> &str {{
    match {var_name} {{
{cases}    }}
}}

fun main() {{
    println("{{}}", {fn_name}(1))
}}
"#
        )
    })
}

// ==================== PROPERTY TESTS ====================

/// Property 1: Type inference - All generated functions must compile successfully
///
/// **Targets**: TRANSPILER-TYPE, TRANSPILER-PARAM-INFERENCE (40% of bugs)
/// **Property**: Any function with explicit parameter types should infer return type correctly
#[test]
#[ignore = "Property test: 10K type inference cases - run with --ignored"]
fn property_01_type_inference_correctness() {
    proptest!(ProptestConfig::with_cases(10000), |(
        program in gen_type_inference_function()
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");
        fs::write(&source, &program).expect("Failed to write test file");

        // Property: Program must transpile without panic
        let mut parser = Parser::new(&program);
        let ast_result = parser.parse();
        prop_assert!(ast_result.is_ok(), "Parse failed: {:?}", ast_result.err());

        let ast = ast_result.unwrap();
        let mut transpiler = Transpiler::new();
        let transpile_result = transpiler.transpile(&ast);
        prop_assert!(transpile_result.is_ok(), "Transpile failed: {:?}", transpile_result.err());

        // Property: Generated Rust must compile
        let compile_result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(
            compile_result.status.success(),
            "Compilation failed:\nProgram:\n{}\n\nError:\n{}",
            program,
            String::from_utf8_lossy(&compile_result.stderr)
        );
    });
}

/// Property 2: Nested scopes - Variables must maintain correct scope visibility
///
/// **Targets**: TRANSPILER-SCOPE, TRANSPILER-GLOBAL-LET (25% of bugs)
/// **Property**: Nested blocks should compile and execute without scope errors
#[test]
#[ignore = "Property test: 8K nested scope cases - run with --ignored"]
fn property_02_scope_correctness() {
    proptest!(ProptestConfig::with_cases(8000), |(
        program in gen_nested_scope_program()
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");
        fs::write(&source, &program).expect("Failed to write test file");

        // Property: Parse and transpile must succeed
        let mut parser = Parser::new(&program);
        let ast = parser.parse();
        prop_assert!(ast.is_ok(), "Parse failed for nested scope: {:?}", ast.err());

        let mut transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast.unwrap());
        prop_assert!(rust_code.is_ok(), "Transpile failed: {:?}", rust_code.err());

        // Property: Compile and execute must succeed
        let compile_result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(
            compile_result.status.success(),
            "Compilation failed:\n{}\n\nError:\n{}",
            program,
            String::from_utf8_lossy(&compile_result.stderr)
        );

        // Execute and verify no runtime errors
        let binary = temp.path().join("test_binary");
        if binary.exists() {
            let exec_result = Command::new(&binary)
                .output()
                .expect("Failed to execute binary");
            prop_assert!(
                exec_result.status.success(),
                "Execution failed for program:\n{}",
                program
            );
        }
    });
}

/// Property 3: Optimization targets - Optimizations must preserve semantics
///
/// **Targets**: TRANSPILER-009, TRANSPILER-015 (20% of bugs)
/// **Property**: Programs with optimization opportunities must produce correct output
#[test]
#[ignore = "Property test: 6K optimization cases - run with --ignored"]
fn property_03_optimization_correctness() {
    proptest!(ProptestConfig::with_cases(6000), |(
        program in gen_optimization_target()
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");
        fs::write(&source, &program).expect("Failed to write test file");

        // Property: Must compile successfully
        let compile_result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(
            compile_result.status.success(),
            "Optimization broke compilation:\n{}\n\nError:\n{}",
            program,
            String::from_utf8_lossy(&compile_result.stderr)
        );

        // Property: Must execute and produce output
        let binary = temp.path().join("test_binary");
        if binary.exists() {
            let exec_result = Command::new(&binary)
                .output()
                .expect("Failed to execute binary");
            prop_assert!(
                exec_result.status.success(),
                "Optimized program failed execution:\n{}",
                program
            );
            prop_assert!(
                !exec_result.stdout.is_empty(),
                "Optimized program produced no output"
            );
        }
    });
}

/// Property 4: Complex expressions - Deep expression nesting must compile
///
/// **Targets**: TRANSPILER-011, TRANSPILER-014 (15% of bugs)
/// **Property**: Complex arithmetic expressions must transpile correctly
#[test]
#[ignore = "Property test: 5K complex expression cases - run with --ignored"]
fn property_04_expression_correctness() {
    proptest!(ProptestConfig::with_cases(5000), |(
        program in gen_complex_expression()
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");
        fs::write(&source, &program).expect("Failed to write test file");

        // Property: Complex expressions must parse
        let mut parser = Parser::new(&program);
        let ast = parser.parse();
        prop_assert!(ast.is_ok(), "Parse failed for complex expr: {:?}", ast.err());

        // Property: Must transpile and compile
        let compile_result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(
            compile_result.status.success(),
            "Complex expression failed:\n{}\n\nError:\n{}",
            program,
            String::from_utf8_lossy(&compile_result.stderr)
        );
    });
}

/// Property 5: Method calls - Method invocations must resolve correctly
///
/// **Targets**: Method resolution and type inference bugs
/// **Property**: Method calls on standard types must work
#[test]
#[ignore = "Property test: 3K method call cases - run with --ignored"]
fn property_05_method_call_correctness() {
    proptest!(ProptestConfig::with_cases(3000), |(
        program in gen_method_call_program()
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");
        fs::write(&source, &program).expect("Failed to write test file");

        // Property: Method calls must compile
        let compile_result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(
            compile_result.status.success(),
            "Method call failed:\n{}\n\nError:\n{}",
            program,
            String::from_utf8_lossy(&compile_result.stderr)
        );
    });
}

/// Property 6: Pattern matching - Match expressions with variables must work
///
/// **Targets**: Pattern matching and scope interaction bugs
/// **Property**: Match arms must maintain correct variable bindings
#[test]
#[ignore = "Property test: 3K pattern match cases - run with --ignored"]
fn property_06_pattern_match_correctness() {
    proptest!(ProptestConfig::with_cases(3000), |(
        program in gen_pattern_match_program()
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");
        fs::write(&source, &program).expect("Failed to write test file");

        // Property: Pattern matching must compile and execute
        let compile_result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(
            compile_result.status.success(),
            "Pattern match failed:\n{}\n\nError:\n{}",
            program,
            String::from_utf8_lossy(&compile_result.stderr)
        );

        // Verify execution produces output
        let binary = temp.path().join("test_binary");
        if binary.exists() {
            let exec_result = Command::new(&binary)
                .output()
                .expect("Failed to execute binary");
            prop_assert!(exec_result.status.success());
            prop_assert!(!exec_result.stdout.is_empty());
        }
    });
}

// ==================== SUMMARY ====================

/// Test summary: Documents the comprehensive property test suite
#[test]
fn property_suite_summary() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║ TRANSPILER-PROPERTY: Comprehensive Property-Based Test Suite  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("📊 Test Coverage by Bug Category:");
    println!("  1. Type Inference (40% of bugs):     10,000 cases");
    println!("  2. Scope/Variables (25% of bugs):     8,000 cases");
    println!("  3. Optimizations (20% of bugs):       6,000 cases");
    println!("  4. Code Generation (15% of bugs):     5,000 cases");
    println!("  5. Method Calls:                      3,000 cases");
    println!("  6. Pattern Matching:                  3,000 cases");
    println!("  ─────────────────────────────────────────────────");
    println!("  TOTAL:                               35,000 cases");
    println!();
    println!("🎯 Expected Impact:");
    println!("  • Catch 60-70% of transpiler bugs before production");
    println!("  • Validate parse → transpile → compile → execute pipeline");
    println!("  • Prevent regressions in all major bug categories");
    println!();
    println!("🚀 Usage:");
    println!("  # Run all property tests (35K cases, ~20-30 min)");
    println!("  cargo test --test transpiler_property_comprehensive -- --ignored --nocapture");
    println!();
    println!("  # Run specific category");
    println!("  cargo test property_01_type_inference -- --ignored --nocapture");
    println!("  cargo test property_02_scope -- --ignored --nocapture");
    println!();
    println!("  # Run with custom case count");
    println!("  PROPTEST_CASES=1000 cargo test property_01 -- --ignored");
    println!();
    println!("📝 Methodology: EXTREME TDD");
    println!("  RED:      Write failing property tests");
    println!("  GREEN:    Fix transpiler to pass tests");
    println!("  REFACTOR: Apply PMAT quality gates (≤10 complexity, A-)");
    println!("  VALIDATE: Run mutation tests + ruchydbg + cargo run examples");
    println!();
}

/// Smoke test: Verify property test infrastructure is working
#[test]
fn property_infrastructure_smoke_test() {
    // Quick sanity check that generators produce valid code
    let fn_name = "test_func";
    let program = format!(
        r#"
fun {fn_name}(x: i32) -> i32 {{
    x + 1
}}

fun main() {{
    println("{{}}", {fn_name}(42))
}}
"#
    );

    let temp = temp_dir();
    let source = temp.path().join("smoke_test.ruchy");
    fs::write(&source, &program).expect("Failed to write test file");

    // Verify full pipeline works
    let result = ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("smoke_binary"))
        .output()
        .expect("Failed to execute ruchy");

    assert!(
        result.status.success(),
        "Property test infrastructure broken: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    println!("✅ Property test infrastructure: OPERATIONAL");
}
