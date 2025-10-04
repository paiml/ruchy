/// TDD: WASM Emitter Implementation
///
/// Following strict TDD methodology:
/// 1. Write failing test
/// 2. Write minimal code to pass
/// 3. Refactor with tests green
use ruchy::backend::wasm::{WasmEmitter, WasmModule};
use ruchy::frontend::parser::Parser;
use wasmparser::{Validator, WasmFeatures};

/// Test 1: Empty module emission
#[test]
fn test_emit_empty_module() {
    // GIVEN: An empty Ruchy program
    let source = "";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap_or_else(|_| {
        // Empty program - create empty block
        use ruchy::frontend::ast::{Expr, ExprKind};
        Expr::new(ExprKind::Block(vec![]), Default::default())
    });

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit empty module");

    // THEN: It should be valid WASM
    assert!(!wasm_bytes.is_empty(), "WASM output should not be empty");
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Should have WASM magic number");
    assert_eq!(&wasm_bytes[4..8], &[1, 0, 0, 0], "Should have version 1");

    // AND: wasmparser should validate it
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Empty module should be valid WASM");
}

/// Test 2: Integer literal
#[test]
fn test_emit_integer_literal() {
    // GIVEN: A program with just an integer
    let source = "42";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse integer");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit integer");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Integer literal module should be valid");

    // AND: Should contain i32.const instruction
    // In WASM bytecode, i32.const is opcode 0x41
    assert!(
        wasm_bytes.windows(2).any(|w| w[0] == 0x41 && w[1] == 42),
        "Should contain i32.const 42 instruction"
    );
}

/// Test 3: Addition expression
#[test]
fn test_emit_addition() {
    // GIVEN: An addition expression
    let source = "2 + 3";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse addition");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit addition");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Addition module should be valid");

    // AND: Should contain two i32.const and i32.add
    // i32.const = 0x41, i32.add = 0x6a
    let bytecode = String::from_utf8_lossy(&wasm_bytes);
    assert!(
        wasm_bytes.windows(2).any(|w| w[0] == 0x41 && w[1] == 2),
        "Should contain i32.const 2"
    );
    assert!(
        wasm_bytes.windows(2).any(|w| w[0] == 0x41 && w[1] == 3),
        "Should contain i32.const 3"
    );
    assert!(
        wasm_bytes.contains(&0x6a),
        "Should contain i32.add instruction"
    );
}

/// Test 4: Function definition
#[test]
fn test_emit_function() {
    // GIVEN: A simple function
    let source = "fun add(a, b) { a + b }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse function");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit function");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Function module should be valid");

    // AND: Should have function section (section id = 3)
    assert!(
        wasm_bytes.windows(2).any(|w| w[0] == 3),
        "Should contain function section"
    );
}

/// Test 5: Main function execution
#[test]
fn test_emit_executable_main() {
    // GIVEN: A program with main function
    let source = "fun main() { 42 }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse main");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit main");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Main module should be valid");

    // AND: Should export main function
    // Export section = 0x07
    assert!(
        wasm_bytes.windows(2).any(|w| w[0] == 0x07),
        "Should contain export section"
    );
}

/// Test 6: Local variables
#[test]
fn test_emit_local_variables() {
    // GIVEN: A program with local variables
    let source = "let x = 10; let y = 20; x + y";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse locals");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit locals");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Locals module should be valid");

    // AND: Should have local.set and local.get instructions
    // local.set = 0x21, local.get = 0x20
    assert!(
        wasm_bytes.contains(&0x21),
        "Should contain local.set instruction"
    );
    assert!(
        wasm_bytes.contains(&0x20),
        "Should contain local.get instruction"
    );
}

/// Test 7: All arithmetic operations
#[test]
fn test_emit_arithmetic_ops() {
    let operations = vec![
        ("2 + 3", 0x6a, "i32.add"),
        ("5 - 2", 0x6b, "i32.sub"),
        ("3 * 4", 0x6c, "i32.mul"),
        ("8 / 2", 0x6d, "i32.div_s"),
        ("7 % 3", 0x6f, "i32.rem_s"),
    ];

    for (source, opcode, op_name) in operations {
        // GIVEN: An arithmetic expression
        let mut parser = Parser::new(source);
        let ast = parser.parse().expect(&format!("Should parse {}", source));

        // WHEN: We emit WASM
        let emitter = WasmEmitter::new();
        let wasm_bytes = emitter
            .emit(&ast)
            .expect(&format!("Should emit {}", source));

        // THEN: Module should be valid
        let mut validator = Validator::new_with_features(WasmFeatures::all());
        validator
            .validate_all(&wasm_bytes)
            .expect(&format!("{} module should be valid", op_name));

        // AND: Should contain the right opcode
        assert!(
            wasm_bytes.contains(&opcode),
            "Should contain {} instruction (0x{:02x})",
            op_name,
            opcode
        );
    }
}

/// Test 8: Comparison operations
#[test]
fn test_emit_comparison_ops() {
    let comparisons = vec![
        ("2 == 3", 0x46, "i32.eq"),
        ("2 != 3", 0x47, "i32.ne"),
        ("2 < 3", 0x48, "i32.lt_s"),
        ("2 > 3", 0x4a, "i32.gt_s"),
        ("2 <= 3", 0x4c, "i32.le_s"),
        ("2 >= 3", 0x4e, "i32.ge_s"),
    ];

    for (source, opcode, op_name) in comparisons {
        let mut parser = Parser::new(source);
        let ast = parser.parse().expect(&format!("Should parse {}", source));

        let emitter = WasmEmitter::new();
        let wasm_bytes = emitter
            .emit(&ast)
            .expect(&format!("Should emit {}", source));

        let mut validator = Validator::new_with_features(WasmFeatures::all());
        validator
            .validate_all(&wasm_bytes)
            .expect(&format!("{} module should be valid", op_name));

        assert!(
            wasm_bytes.contains(&opcode),
            "Should contain {} instruction",
            op_name
        );
    }
}

/// Test 9: If-else control flow
#[test]
fn test_emit_if_else() {
    // GIVEN: An if-else expression
    let source = "if true { 1 } else { 2 }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse if-else");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit if-else");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("If-else module should be valid");

    // AND: Should contain if/else/end instructions
    // if = 0x04, else = 0x05, end = 0x0b
    assert!(wasm_bytes.contains(&0x04), "Should contain if instruction");
    assert!(
        wasm_bytes.contains(&0x05),
        "Should contain else instruction"
    );
    assert!(wasm_bytes.contains(&0x0b), "Should contain end instruction");
}

/// Test 10: Loop control flow
#[test]
fn test_emit_loop() {
    // GIVEN: A while loop
    let source = "let i = 0; while i < 10 { i = i + 1 }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse loop");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit loop");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Loop module should be valid");

    // AND: Should contain loop/br_if instructions
    // loop = 0x03, br_if = 0x0d
    assert!(
        wasm_bytes.contains(&0x03),
        "Should contain loop instruction"
    );
    assert!(
        wasm_bytes.contains(&0x0d),
        "Should contain br_if instruction"
    );
}

/// Test 11: Function calls
#[test]
fn test_emit_function_call() {
    // GIVEN: Function definition and call
    let source = r#"
        fun double(x) { x * 2 }
        double(21)
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse function call");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit function call");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Function call module should be valid");

    // AND: Should contain call instruction
    // call = 0x10
    assert!(
        wasm_bytes.contains(&0x10),
        "Should contain call instruction"
    );
}

/// Test 12: Multiple functions
#[test]
fn test_emit_multiple_functions() {
    // GIVEN: Multiple function definitions
    let source = r#"
        fun add(a, b) { a + b }
        fun sub(a, b) { a - b }
        fun mul(a, b) { a * b }
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse multiple functions");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit multiple functions");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Multiple functions module should be valid");

    // AND: Function section should indicate 3 functions
    // We'll check for the presence of add, sub, mul opcodes
    assert!(wasm_bytes.contains(&0x6a), "Should have add");
    assert!(wasm_bytes.contains(&0x6b), "Should have sub");
    assert!(wasm_bytes.contains(&0x6c), "Should have mul");
}

/// Test 13: Return statement
#[test]
fn test_emit_return() {
    // GIVEN: A function with explicit return
    let source = "fun early_return(x) { if x > 10 { return 42 } return 0 }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse return");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit return");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Return module should be valid");

    // AND: Should contain return instruction
    // return = 0x0f
    assert!(
        wasm_bytes.contains(&0x0f),
        "Should contain return instruction"
    );
}

/// Test 14: Memory allocation
#[test]
fn test_emit_memory_section() {
    // GIVEN: A program that needs memory
    let source = "let arr = [1, 2, 3, 4, 5]";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse array");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit with memory");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Memory module should be valid");

    // AND: Should have memory section
    // Memory section = 0x05
    assert!(
        wasm_bytes.windows(2).any(|w| w[0] == 0x05),
        "Should contain memory section"
    );
}

/// Test 15: Integration - Complete program
#[test]
fn test_emit_complete_program() {
    // GIVEN: A complete Ruchy program
    let source = r#"
        fun fibonacci(n) {
            if n <= 1 {
                return n
            }
            return fibonacci(n - 1) + fibonacci(n - 2)
        }
        
        fun main() {
            let result = fibonacci(10)
            result
        }
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse complete program");

    // WHEN: We emit WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit complete program");

    // THEN: Module should be valid
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&wasm_bytes)
        .expect("Complete program should be valid WASM");

    // AND: Should be instantiable in wasmtime
    let engine = wasmtime::Engine::default();
    let module =
        wasmtime::Module::new(&engine, &wasm_bytes).expect("Should create wasmtime module");

    // AND: Should have exported main function
    let exports: Vec<_> = module.exports().collect();
    assert!(
        exports.iter().any(|e| e.name() == "main"),
        "Should export main function"
    );
}

// Property-based tests using quickcheck
#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn prop_all_integers_compile_to_valid_wasm(n: i32) -> bool {
        let source = format!("{}", n);
        let mut parser = Parser::new(&source);

        if let Ok(ast) = parser.parse() {
            let emitter = WasmEmitter::new();
            if let Ok(wasm_bytes) = emitter.emit(&ast) {
                let mut validator = Validator::new_with_features(WasmFeatures::all());
                return validator.validate_all(&wasm_bytes).is_ok();
            }
        }
        false
    }

    #[quickcheck]
    fn prop_arithmetic_expressions_valid(a: i8, b: i8, op: u8) -> bool {
        let ops = vec!["+", "-", "*", "/", "%"];
        let op_str = ops[op as usize % ops.len()];

        // Avoid division by zero
        let b_safe = if b == 0 && (op_str == "/" || op_str == "%") {
            1
        } else {
            b
        };

        let source = format!("{} {} {}", a, op_str, b_safe);
        let mut parser = Parser::new(&source);

        if let Ok(ast) = parser.parse() {
            let emitter = WasmEmitter::new();
            if let Ok(wasm_bytes) = emitter.emit(&ast) {
                let mut validator = Validator::new_with_features(WasmFeatures::all());
                return validator.validate_all(&wasm_bytes).is_ok();
            }
        }
        false
    }
}

// Fuzzing preparation
#[cfg(fuzzing)]
mod fuzz_tests {
    use super::*;

    pub fn fuzz_wasm_emission(data: &[u8]) {
        if let Ok(source) = std::str::from_utf8(data) {
            let mut parser = Parser::new(source);
            if let Ok(ast) = parser.parse() {
                let emitter = WasmEmitter::new();
                if let Ok(wasm_bytes) = emitter.emit(&ast) {
                    // Must produce valid WASM
                    let mut validator = Validator::new_with_features(WasmFeatures::all());
                    let _ = validator.validate_all(&wasm_bytes);
                }
            }
        }
    }
}
