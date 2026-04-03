# Sub-spec: SQLite-Style Testing — Code Generation & Transpilation

**Parent:** [ruchy-sqlite-testing-v2.md](../ruchy-sqlite-testing-v2.md) Section 1.3

---

### 1.3 Backend: Code Generation & Transpilation

**SQLite Equivalent**: VDBE bytecode generator  
**Ruchy Standard**: Semantic equivalence via metamorphic testing

#### Theoretical Foundation: Metamorphic Testing

**Research Grounding**: 
> Chen, T. Y., Kuo, F. C., Liu, H., & Poon, P. L. (2018). "Metamorphic testing: A review of challenges and opportunities". ACM Computing Surveys (CSUR), 51(1), 1-27.

**Key Innovation**: Metamorphic Testing (MT) addresses the oracle problem—when expected output is unknown, how do you test? MT defines **Metamorphic Relations** (MRs): properties that must hold when transforming inputs.

For compilers: If `P` is a program and `Optimize(P)` is its optimized version, then the metamorphic relation is:
```
MR: Execute(P) ≡ Execute(Optimize(P))
```

This allows generating near-infinite test cases by applying transformations and verifying equivalence.

#### Test Harness 1.3: Metamorphic Code Generation Validation

```rust
// tests/codegen_metamorphic.rs

/**
 * Metamorphic Testing for Code Generation
 * 
 * Research Foundation:
 * Citation: Chen, T. Y., Kuo, F. C., Liu, H., & Poon, P. L. (2018).
 * Metamorphic testing: A review of challenges and opportunities.
 * ACM Computing Surveys (CSUR), 51(1), 1-27.
 * 
 * Metamorphic Relations (MRs) define properties that must hold
 * when transforming inputs. For compilers, we define 6 critical MRs:
 * 
 * MR1: Optimization Equivalence
 * MR2: Statement Permutation
 * MR3: Constant Propagation
 * MR4: Alpha Renaming
 * MR5: Interpreter-Compiler Equivalence
 * MR6: Parse-Print-Parse Identity
 */

#[cfg(test)]
mod metamorphic_relations {
    use proptest::prelude::*;
    use ruchy::*;
    
    // ========================================================================
    // MR1: Optimization Preserves Semantics
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100_000))]
        
        #[test]
        fn mr1_optimization_equivalence(prog in any_program()) {
            /**
             * Metamorphic Relation 1: Optimization Equivalence
             * 
             * Property: Optimize(P) ≡ P
             * 
             * Compiler optimizations must preserve program semantics.
             * Examples:
             * - Constant folding: 2 + 3 → 5
             * - Dead code elimination: if false { x } → ε
             * - Common subexpression elimination: a+b, a+b → let t=a+b; t, t
             */
            
            let ast = parse(&prog).unwrap();
            let optimized_ast = optimize(&ast);
            
            let output_original = interpret(&ast);
            let output_optimized = interpret(&optimized_ast);
            
            assert_eq!(
                normalize_output(output_original),
                normalize_output(output_optimized),
                "MR1 violated: Optimization changed semantics\n  \
                 Original program: {}\n  \
                 Optimized program: {}",
                prog, pretty_print(&optimized_ast)
            );
        }
    }
    
    // ========================================================================
    // MR2: Statement Permutation Equivalence
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50_000))]
        
        #[test]
        fn mr2_statement_permutation(prog in program_with_independent_statements()) {
            /**
             * Metamorphic Relation 2: Statement Permutation
             * 
             * Property: If statements S1 and S2 are independent (no data flow),
             *           then [S1; S2] ≡ [S2; S1]
             * 
             * Example:
             *   let x = 1; let y = 2;  ≡  let y = 2; let x = 1;
             * 
             * This tests that the compiler correctly handles statement ordering
             * and doesn't introduce spurious dependencies.
             */
            
            let permuted = permute_independent_statements(&prog);
            
            let state1 = interpret(&prog).final_memory_state;
            let state2 = interpret(&permuted).final_memory_state;
            
            assert_eq!(
                normalize_state(state1),
                normalize_state(state2),
                "MR2 violated: Permuting independent statements changed result\n  \
                 Original: {}\n  \
                 Permuted: {}",
                prog, permuted
            );
        }
    }
    
    // ========================================================================
    // MR3: Constant Propagation Correctness
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50_000))]
        
        #[test]
        fn mr3_constant_propagation(prog in program_with_constants()) {
            /**
             * Metamorphic Relation 3: Constant Propagation
             * 
             * Property: If variable x is assigned constant c and never modified,
             *           replacing all uses of x with c preserves semantics.
             * 
             * Example:
             *   let x = 42; let y = x + 1;  ≡  let x = 42; let y = 42 + 1;
             */
            
            let propagated = propagate_constants(&prog);
            
            assert_eq!(
                interpret(&prog),
                interpret(&propagated),
                "MR3 violated: Constant propagation changed semantics\n  \
                 Original: {}\n  \
                 Propagated: {}",
                prog, propagated
            );
        }
    }
    
    // ========================================================================
    // MR4: Alpha Renaming (Variable Renaming)
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50_000))]
        
        #[test]
        fn mr4_alpha_renaming(prog in any_program()) {
            /**
             * Metamorphic Relation 4: Alpha Equivalence
             * 
             * Property: Renaming bound variables preserves semantics.
             * 
             * Example:
             *   λx. x + 1  ≡  λy. y + 1
             * 
             * This tests that the compiler correctly implements lexical scoping
             * and variable capture.
             */
            
            let renamed = alpha_rename_all_bound_vars(&prog);
            
            assert_eq!(
                interpret(&prog),
                interpret(&renamed),
                "MR4 violated: Alpha renaming changed semantics\n  \
                 Original: {}\n  \
                 Renamed: {}",
                prog, renamed
            );
        }
    }
    
    // ========================================================================
    // MR5: Interpreter-Compiler Equivalence (Differential Testing)
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100_000))]
        
        #[test]
        fn mr5_interpreter_compiler_equivalence(prog in any_program()) {
            /**
             * Metamorphic Relation 5: Compilation Correctness
             * 
             * Property: Interpret(P) ≡ Execute(Compile(P))
             * 
             * This is the fundamental compiler correctness property:
             * compiled code must behave identically to interpreted code.
             * 
             * This is also called "differential testing" when comparing
             * against a reference implementation.
             */
            
            let interpreted_output = interpret(&prog);
            
            let rust_code = transpile_to_rust(&prog);
            let compiled_output = compile_and_execute_rust(&rust_code);
            
            assert_eq!(
                normalize_output(interpreted_output),
                normalize_output(compiled_output),
                "MR5 violated: Compiled output diverged from interpreter\n  \
                 Program: {}\n  \
                 Interpreted: {:?}\n  \
                 Compiled: {:?}",
                prog, interpreted_output, compiled_output
            );
        }
    }
    
    // ========================================================================
    // MR6: Parse-Print-Parse Identity
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50_000))]
        
        #[test]
        fn mr6_parse_print_parse_identity(prog in valid_program()) {
            /**
             * Metamorphic Relation 6: Textual Representation Equivalence
             * 
             * Property: parse(pretty_print(parse(P))) = parse(P)
             * 
             * The AST→text→AST roundtrip should be idempotent.
             * This ensures the pretty printer is semantics-preserving.
             */
            
            let ast1 = parse(&prog).unwrap();
            let printed = pretty_print(&ast1);
            let ast2 = parse(&printed).unwrap();
            
            assert_eq!(
                ast1, ast2,
                "MR6 violated: Parse-print-parse not idempotent\n  \
                 Original: {}\n  \
                 Printed: {}\n  \
                 AST1: {:?}\n  \
                 AST2: {:?}",
                prog, printed, ast1, ast2
            );
        }
    }
}

// ============================================================================
// Differential Testing Against Multiple References
// ============================================================================

#[cfg(test)]
mod differential_testing {
    use super::*;
    
    struct ReferenceImplementation {
        name: &'static str,
        execute: fn(&str) -> Result<String, String>,
    }
    
    fn get_reference_implementations() -> Vec<ReferenceImplementation> {
        vec![
            ReferenceImplementation {
                name: "Python (Reference Semantics)",
                execute: execute_via_python,
            },
            ReferenceImplementation {
                name: "Ruby (Alternative Semantics)",
                execute: execute_via_ruby,
            },
            ReferenceImplementation {
                name: "Ruchy v0.9 (Regression Check)",
                execute: execute_via_previous_version,
            },
        ]
    }
    
    #[test]
    fn differential_validation_100k_programs() {
        /**
         * Large-Scale Differential Testing
         * 
         * Generate 100,000 random programs and validate that:
         * 1. Ruchy output matches Python semantics
         * 2. Ruchy output matches Ruby semantics (where applicable)
         * 3. No regressions from previous Ruchy version
         * 
         * Inspired by SQLite's SLT: 7.2M queries against 4 databases.
         */
        
        let test_programs = generate_diverse_test_programs(100_000);
        let references = get_reference_implementations();
        
        let mut divergences = Vec::new();
        
        for (i, program) in test_programs.iter().enumerate() {
            if i % 1000 == 0 {
                println!("Validated {} / {} programs", i, test_programs.len());
            }
            
            let ruchy_result = execute_current_ruchy(program);
            
            for reference in &references {
                let ref_result = (reference.execute)(program);
                
                if !results_equivalent(&ruchy_result, &ref_result) {
                    divergences.push(Divergence {
                        program_id: i,
                        program: program.clone(),
                        reference_name: reference.name,
                        ruchy_output: ruchy_result.clone(),
                        reference_output: ref_result.clone(),
                    });
                }
            }
        }
        
        // Allow up to 10 divergences (edge cases in reference implementations)
        assert!(
            divergences.len() < 10,
            "Too many divergences found: {}\n{:#?}",
            divergences.len(),
            &divergences[0..divergences.len().min(5)]
        );
    }
    
    #[test]
    fn regression_detection() {
        /**
         * Regression Testing
         * 
         * Load all test cases from previous releases and verify
         * that behavior hasn't changed (unless intentionally modified).
         */
        
        let regression_suite = load_historical_test_suite();
        
        for test in regression_suite {
            let current_output = execute_current_ruchy(&test.program);
            
            assert_eq!(
                normalize_output(current_output),
                normalize_output(test.expected_output.clone()),
                "Regression detected in test '{}': {}\n  \
                 Expected: {:?}\n  \
                 Got: {:?}",
                test.name, test.program,
                test.expected_output, current_output
            );
        }
    }
}

// ============================================================================
// Code Generation Pattern Validation
// ============================================================================

#[cfg(test)]
mod codegen_patterns {
    use super::*;
    
    #[test]
    fn test_variable_codegen() {
        assert_transpiles_to(
            "let x = 42",
            "let x: i64 = 42;"
        );
        
        assert_transpiles_to(
            "let mut x = 42",
            "let mut x: i64 = 42;"
        );
    }
    
    #[test]
    fn test_function_codegen() {
        assert_transpiles_to(
            "fun add(a, b) { a + b }",
            "fn add(a: i64, b: i64) -> i64 {\n    a + b\n}"
        );
        
        // Generic functions
        assert_transpiles_to(
            "fun identity<T>(x: T) { x }",
            "fn identity<T>(x: T) -> T {\n    x\n}"
        );
    }
    
    #[test]
    fn test_closure_codegen() {
        assert_transpiles_to(
            "fun make_adder(n) { fun(x) { x + n } }",
            "fn make_adder(n: i64) -> impl Fn(i64) -> i64 {\n    \
               move |x| x + n\n}"
        );
    }
    
    #[test]
    fn test_pattern_match_codegen() {
        assert_transpiles_to(
            r#"match x {
                Some(y) => y,
                None => 0
            }"#,
            r#"match x {
                Some(y) => y,
                None => 0,
            }"#
        );
        
        // Exhaustiveness should be preserved
        assert_transpiles_to(
            "match x { 1 | 2 | 3 => \"small\", _ => \"large\" }",
            "match x {\n    1 | 2 | 3 => \"small\",\n    _ => \"large\",\n}"
        );
    }
    
    #[test]
    fn test_error_handling_codegen() {
        // ? operator
        assert_transpiles_to(
            "let x = may_fail()?",
            "let x = may_fail()?;"
        );
        
        // Result type
        assert_transpiles_to(
            "fun risky() -> Result<Int, String> { Ok(42) }",
            "fn risky() -> Result<i64, String> {\n    Ok(42)\n}"
        );
    }
}

// ============================================================================
// Memory Safety Validation
// ============================================================================

#[cfg(test)]
mod memory_safety {
    use super::*;
    
    #[test]
    fn test_borrow_checker_integration() {
        /**
         * Verify that generated Rust code respects borrow checker rules.
         * 
         * Ruchy's type system must ensure that compiled Rust code
         * never violates Rust's ownership/borrowing rules.
         */
        
        let programs_that_should_compile = [
            r#"
            let x = [1, 2, 3];
            let y = &x;
            print(y);
            "#,
            r#"
            let mut x = 42;
            x = x + 1;
            print(x);
            "#,
        ];
        
        for prog in programs_that_should_compile {
            let rust_code = transpile_to_rust(prog);
            let compile_result = attempt_rust_compilation(&rust_code);
            
            assert!(
                compile_result.is_ok(),
                "Valid Ruchy program should generate valid Rust:\n{}\n\nRust error: {}",
                prog, compile_result.unwrap_err()
            );
        }
    }
    
    #[test]
    fn test_use_after_move_prevented() {
        let programs_with_move_errors = [
            r#"
            let x = vec![1, 2, 3];
            let y = x;
            print(x);  // Error: x moved to y
            "#,
        ];
        
        for prog in programs_with_move_errors {
            let result = type_check(prog);
            
            assert!(
                result.is_err(),
                "Ruchy type checker should catch move errors: {}",
                prog
            );
            
            assert!(
                result.unwrap_err().contains("moved") ||
                result.unwrap_err().contains("borrow"),
                "Error should mention move/borrow violation"
            );
        }
    }
    
    #[test]
    fn test_lifetime_inference() {
        /**
         * Test that Ruchy correctly infers Rust lifetimes for references.
         */
        
        let prog = r#"
        fun first<T>(list: &[T]) -> &T {
            &list[0]
        }
        "#;
        
        let rust_code = transpile_to_rust(prog);
        
        // Should generate correct lifetime annotations
        assert!(rust_code.contains("fn first<'a, T>(list: &'a [T]) -> &'a T"));
        
        // Rust compiler should accept it
        assert!(attempt_rust_compilation(&rust_code).is_ok());
    }
}
```

**Coverage Target**:
- 100% codegen paths
- 100,000+ metamorphic test iterations
- 100,000+ differential tests against 3 references
- <10 divergences tolerated

**Test Count**: 200,000+ codegen tests

---
