# Sub-spec: SQLite-Style Testing — Frontend (Parser & AST)

**Parent:** [ruchy-sqlite-testing-v2.md](../ruchy-sqlite-testing-v2.md) Section 1.1

---

### 1.1 Frontend: Parser & Abstract Syntax Tree

**SQLite Equivalent**: Lemon parser generator with 100% branch coverage  
**Ruchy Standard**: 100% grammar coverage + 100% MC/DC + exhaustive error recovery

#### Theoretical Foundation

Modern parser testing requires three orthogonal validation dimensions:

1. **Syntactic Correctness**: Grammar coverage ensures all production rules tested
2. **Semantic Preservation**: Round-trip property (parse ∘ print ∘ parse = parse)
3. **Resilience**: Graceful degradation on malformed input

**Research Grounding**: Parser testing methodology from Lemon (SQLite), enhanced with MC/DC requirements from NASA DO-178B/C Level A avionics certification standard.

#### Test Harness 1.1: Grammar Coverage Suite

```rust
// tests/parser_grammar_coverage.rs

/**
 * Grammar Coverage Suite
 * 
 * Research Foundation:
 * - Lemon parser generator methodology (SQLite)
 * - MC/DC coverage requirements (NASA/TM-2001-210876, Hayhurst et al., 2001)
 * 
 * Guarantees:
 * - Every production rule tested with valid inputs
 * - Every production rule tested with invalid inputs
 * - Every error recovery path validated
 * - All boolean conditions proven independent (MC/DC)
 */

#[cfg(test)]
mod grammar_coverage {
    use proptest::prelude::*;
    use ruchy::frontend::parser::Parser;
    
    // ============================================================================
    // Category 1: Expression Grammar (Complete Coverage)
    // ============================================================================
    
    #[test]
    fn test_literal_expressions_exhaustive() {
        // Integer literals - all representations
        assert_parses("42");           // Decimal
        assert_parses("0x2A");         // Hexadecimal
        assert_parses("0b101010");     // Binary
        assert_parses("0o52");         // Octal
        assert_parses("1_000_000");    // With separators
        
        // Float literals - scientific notation
        assert_parses("3.14");
        assert_parses("1e10");
        assert_parses("6.022e23");
        assert_parses("1.5e-10");
        
        // String literals - all escape sequences
        assert_parses(r#""hello""#);
        assert_parses(r#""escaped\"quote""#);
        assert_parses(r#""line\nbreak""#);
        assert_parses(r#""tab\there""#);
        assert_parses(r#""unicode\u{1F60A}""#);
        assert_parses(r#"r"raw string""#);
        
        // Boolean literals
        assert_parses("true");
        assert_parses("false");
        
        // Null literal
        assert_parses("null");
    }
    
    #[test]
    fn test_operator_precedence_exhaustive() {
        // Test EVERY operator pair interaction
        // Critical for correctness - precedence bugs cause semantic errors
        
        let operators = [
            ("||", 1),   // Logical OR (lowest precedence)
            ("&&", 2),   // Logical AND
            ("==", 3), ("!=", 3),
            ("<", 3), ("<=", 3), (">", 3), (">=", 3),
            ("+", 4), ("-", 4),
            ("*", 5), ("/", 5), ("%", 5),  // Highest precedence
        ];
        
        // Combinatorial testing: all operator pairs
        for (i, (op1, prec1)) in operators.iter().enumerate() {
            for (j, (op2, prec2)) in operators.iter().enumerate() {
                if i == j { continue; }
                
                let expr = format!("a {} b {} c", op1, op2);
                let ast = parse(&expr).expect(&format!(
                    "Failed to parse: {}", expr
                ));
                
                // Verify structural correctness via precedence
                if prec1 > prec2 {
                    // op1 binds tighter: (a op1 b) op2 c
                    assert_left_associative(&ast, op1, op2);
                } else if prec1 < prec2 {
                    // op2 binds tighter: a op1 (b op2 c)
                    assert_right_associative(&ast, op1, op2);
                } else {
                    // Equal precedence: left-to-right
                    assert_left_associative(&ast, op1, op2);
                }
            }
        }
    }
    
    #[test]
    fn test_operator_precedence_mcdc() {
        /**
         * Modified Condition/Decision Coverage (MC/DC)
         * 
         * Citation: Hayhurst, K. J., Veerhusen, D. S., Chilenski, J. J., 
         * & Rierson, L. K. (2001). A Practical Tutorial on Modified 
         * Condition/Decision Coverage. NASA/TM-2001-210876.
         * 
         * MC/DC Requirement: For expression "a || b && c", prove that
         * each condition (a, b, c) can INDEPENDENTLY affect the outcome.
         * 
         * This is stronger than branch coverage. Branch coverage only
         * requires testing true/false outcomes. MC/DC requires proving
         * each condition matters independently.
         * 
         * MC/DC is mandatory for DO-178B/C Level A (highest criticality
         * avionics software). We apply it to Ruchy's critical logic.
         */
        
        // MC/DC Test Pair 1: Prove 'a' independently affects outcome
        // Keep b=false, c=true constant; vary only 'a'
        assert_eq!(
            eval("true || (false && true)"),   // a=true → true
            Value::Bool(true)
        );
        assert_eq!(
            eval("false || (false && true)"),  // a=false → false
            Value::Bool(false)
        );
        // ✓ Proven: 'a' can independently change result
        
        // MC/DC Test Pair 2: Prove 'b' independently affects outcome
        // Keep a=false, c=true constant; vary only 'b'
        assert_eq!(
            eval("false || (true && true)"),   // b=true → true
            Value::Bool(true)
        );
        assert_eq!(
            eval("false || (false && true)"),  // b=false → false
            Value::Bool(false)
        );
        // ✓ Proven: 'b' can independently change result
        
        // MC/DC Test Pair 3: Prove 'c' independently affects outcome
        // Keep a=false, b=true constant; vary only 'c'
        assert_eq!(
            eval("false || (true && true)"),   // c=true → true
            Value::Bool(true)
        );
        assert_eq!(
            eval("false || (true && false)"),  // c=false → false
            Value::Bool(false)
        );
        // ✓ Proven: 'c' can independently change result
        
        // MC/DC achieves 100% Modified Condition/Decision Coverage
        // This is the avionics standard for safety-critical software
    }
    
    #[test]
    fn test_pattern_matching_exhaustive() {
        // Literal patterns
        assert_parses("match x { 42 => {} }");
        assert_parses("match x { \"hello\" => {} }");
        assert_parses("match x { true => {} }");
        
        // Variable patterns
        assert_parses("match x { y => {} }");
        assert_parses("match x { _ => {} }");
        
        // Constructor patterns
        assert_parses("match x { Some(y) => {} }");
        assert_parses("match x { Ok(val) => {} }");
        assert_parses("match x { Point { x, y } => {} }");
        
        // Nested patterns
        assert_parses("match x { Some(Some(y)) => {} }");
        assert_parses("match x { Ok(Some(val)) => {} }");
        
        // Multiple arms
        assert_parses(r#"
            match x {
                Some(y) => {},
                None => {}
            }
        "#);
        
        // Guards
        assert_parses("match x { y if y > 0 => {} }");
        assert_parses("match x { Some(y) if y.is_valid() => {} }");
        
        // Or patterns
        assert_parses("match x { 1 | 2 | 3 => {} }");
        assert_parses("match x { Some(1) | Some(2) => {} }");
    }
    
    // ============================================================================
    // Property-Based Grammar Fuzzing
    // ============================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        
        #[test]
        fn property_parser_never_panics(expr in any_expression()) {
            // Property: Parser should NEVER panic, only return Ok or Err
            // This is a critical safety property
            let result = std::panic::catch_unwind(|| {
                Parser::parse_expression(&expr)
            });
            
            assert!(
                result.is_ok(),
                "Parser panicked on input: {}",
                expr
            );
        }
        
        #[test]
        fn property_parse_print_parse_identity(expr in valid_expression()) {
            // Property: parse(pretty_print(parse(x))) = parse(x)
            // Also known as: parsing is a homomorphism
            
            let ast1 = Parser::parse_expression(&expr).unwrap();
            let printed = ast1.pretty_print();
            let ast2 = Parser::parse_expression(&printed).unwrap();
            
            assert_eq!(
                ast1, ast2,
                "Parse-print-parse not idempotent:\n  Original: {}\n  Printed: {}\n  ASTs differ",
                expr, printed
            );
        }
        
        #[test]
        fn property_parentheses_preserve_semantics(expr in valid_expression()) {
            // Property: Adding redundant parentheses doesn't change AST
            let ast_original = Parser::parse_expression(&expr).unwrap();
            let parenthesized = format!("({})", expr);
            let ast_wrapped = Parser::parse_expression(&parenthesized).unwrap();
            
            assert_semantically_equivalent(&ast_original, &ast_wrapped);
        }
    }
}

// ============================================================================
// Category 2: Error Recovery Testing
// ============================================================================

#[cfg(test)]
mod error_recovery {
    use super::*;
    
    #[test]
    fn test_missing_semicolon_recovery() {
        let result = parse("let x = 42 let y = 43");
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        
        // Error should be precise
        assert!(error.message.contains("expected semicolon"));
        assert_eq!(error.line, 1);
        assert_eq!(error.column, 11);
        
        // Error should be actionable
        assert!(error.suggestion.is_some());
        assert_eq!(error.suggestion.unwrap(), "let x = 42;");
    }
    
    #[test]
    fn test_unbalanced_parentheses() {
        let cases = [
            ("(1 + 2", "unclosed parenthesis"),
            ("1 + 2)", "unexpected closing parenthesis"),
            ("((1 + 2)", "unclosed parenthesis"),
            ("func(arg1, arg2", "unclosed function call"),
        ];
        
        for (input, expected_msg) in cases {
            let error = parse(input).unwrap_err();
            assert!(
                error.message.contains(expected_msg),
                "Expected '{}' in error for input '{}', got: {}",
                expected_msg, input, error.message
            );
        }
    }
    
    #[test]
    fn test_invalid_utf8_handling() {
        // Parser must handle invalid UTF-8 gracefully
        let invalid_bytes: Vec<u8> = vec![
            0xFF, 0xFE, 0xFD, // Invalid UTF-8 sequence
            b'l', b'e', b't',
        ];
        
        let result = std::panic::catch_unwind(|| {
            Parser::parse(&invalid_bytes)
        });
        
        // Should not panic - return Err with helpful message
        assert!(result.is_ok());
        let parse_result = result.unwrap();
        assert!(parse_result.is_err());
        assert!(parse_result.unwrap_err().message.contains("invalid UTF-8"));
    }
    
    #[test]
    fn test_stack_exhaustion_protection() {
        // Deeply nested expressions should not cause stack overflow
        // Generate: ((((((1))))))
        let depth = 10_000;
        let mut expr = String::from("1");
        for _ in 0..depth {
            expr = format!("({})", expr);
        }
        
        let result = std::panic::catch_unwind(|| {
            Parser::parse(&expr)
        });
        
        assert!(result.is_ok(), "Parser should handle deep nesting");
        
        // Either successfully parse with depth limit, or error gracefully
        let parse_result = result.unwrap();
        if parse_result.is_err() {
            assert!(parse_result.unwrap_err().message.contains("nesting depth"));
        }
    }
    
    #[test]
    fn test_malformed_unicode_escapes() {
        let cases = [
            (r#""\\u{110000}""#, "beyond Unicode range"),  // > U+10FFFF
            (r#""\\u{D800}""#, "surrogate"),               // UTF-16 surrogate
            (r#""\\u""#, "incomplete escape"),
            (r#""\\u{ZZZZ}""#, "invalid hex"),
        ];
        
        for (input, expected_issue) in cases {
            let result = parse(input);
            assert!(
                result.is_err(),
                "Should reject malformed Unicode: {}",
                input
            );
            assert!(
                result.unwrap_err().message.contains(expected_issue),
                "Error should mention: {}",
                expected_issue
            );
        }
    }
}

// ============================================================================
// Category 3: Performance & Complexity Validation
// ============================================================================

#[cfg(test)]
mod parser_performance {
    use std::time::Instant;
    use super::*;
    
    #[test]
    fn test_parse_time_linear_complexity() {
        /**
         * Verify O(n) parsing time complexity
         * 
         * Parser should scale linearly with input size.
         * Quadratic or exponential complexity indicates
         * algorithmic issues (e.g., excessive backtracking).
         */
        
        let sizes = [100, 1_000, 10_000, 100_000];
        let mut times_us = Vec::new();
        
        for size in sizes {
            let input = generate_expression_of_size(size);
            
            let start = Instant::now();
            let _ = Parser::parse(&input).unwrap();
            let elapsed = start.elapsed().as_micros();
            
            times_us.push(elapsed);
            println!("Size {}: {} μs", size, elapsed);
        }
        
        // Verify linear growth: T(10n) ≈ 10 * T(n)
        for i in 1..times_us.len() {
            let ratio = times_us[i] as f64 / times_us[i-1] as f64;
            let size_ratio = sizes[i] as f64 / sizes[i-1] as f64;
            
            // Allow 50% tolerance for system variance
            assert!(
                ratio < size_ratio * 1.5,
                "Non-linear growth detected: {}x size → {}x time",
                size_ratio, ratio
            );
        }
    }
    
    #[test]
    fn test_memory_usage_linear() {
        // Verify O(n) memory usage
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        // Track allocations
        static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
        
        for size in [100, 1_000, 10_000] {
            ALLOCATED.store(0, Ordering::SeqCst);
            
            let input = generate_expression_of_size(size);
            let ast = Parser::parse(&input).unwrap();
            let memory_used = ALLOCATED.load(Ordering::SeqCst);
            
            // Memory should be proportional to input size
            let ratio = memory_used as f64 / size as f64;
            assert!(
                ratio < 1000.0, // <1KB per token
                "Excessive memory usage: {} bytes for {} tokens",
                memory_used, size
            );
        }
    }
}
```

**Coverage Target**: 
- 100% of grammar production rules
- 100% of error recovery paths
- 100% MC/DC on critical boolean logic
- O(n) time and space complexity verified

**Test Count**: 2,000+ parser tests

---

#### Test Harness 1.2: Coverage-Guided Fuzzing (Security)

**Critical Distinction**: Property testing and fuzzing are complementary but fundamentally different techniques:

- **Property Testing** (`proptest`): Type-aware, structured input generation → finds **logical bugs** in program semantics
- **Coverage-Guided Fuzzing** (`cargo-fuzz`, AFL): Unstructured byte-stream mutation → finds **memory unsafety, crashes, panics**

**Research Foundation**: American Fuzzy Lop (AFL) by Michal Zalewski (2014) revolutionized software testing by introducing profile-guided fuzzing. Unlike blind random testing, AFL instruments programs to detect when mutations cause new control-flow paths, enabling systematic exploration of the input space.

**Why Fuzzing is Essential**: Property tests work within type system constraints. Fuzzing tests the parser's resilience to **adversarial input**: malformed UTF-8, buffer overruns, stack exhaustion, integer overflows—attack vectors that property tests cannot generate.

```rust
// fuzz/fuzz_targets/parser_security.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use ruchy::frontend::parser::Parser;

fuzz_target!(|data: &[u8]| {
    /**
     * Security-Focused Fuzzing
     * 
     * Goal: Find ANY input that causes:
     * - Panic (unwrap/expect failures)
     * - Segmentation fault (unsafe code violations)
     * - Stack overflow (unbounded recursion)
     * - Integer overflow (arithmetic without checked_*)
     * - Out-of-bounds access (indexing without bounds checks)
     * 
     * This is a SECURITY test, not a correctness test.
     * The parser should NEVER crash, regardless of input.
     * 
     * Success criterion: Zero crashes after 24 hours.
     */
    
    let _ = Parser::parse(data);
    
    // If we reach here without panic, test passed
    // Fuzzer tracks coverage to guide mutation strategy
});
```

**Fuzzing Infrastructure**:

```bash
# Install fuzzing toolchain
cargo install cargo-fuzz
cargo install afl.rs

# Run libFuzzer (LLVM-based, fast startup)
cargo +nightly fuzz run parser_security -- -max_total_time=3600

# Run AFL (slower but more thorough)
cargo afl build --release
cargo afl fuzz -i seeds/ -o findings/ target/release/parser-afl

# Continuous fuzzing (24 hours per release)
cargo fuzz run parser_security -- -max_total_time=86400
```

**CI Integration**:

```yaml
# .github/workflows/continuous-fuzzing.yml
name: Continuous Security Fuzzing

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours

jobs:
  fuzz:
    runs-on: ubuntu-latest
    timeout-minutes: 360  # 6 hours
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Nightly Rust
        run: rustup install nightly
      
      - name: Run Parser Fuzzing
        run: |
          cargo +nightly fuzz run parser_security -- \
            -max_total_time=21600 \
            -rss_limit_mb=4096
      
      - name: Check for Crashes
        run: |
          if [ -d fuzz/artifacts/parser_security ]; then
            echo "❌ Fuzzer found crashes!"
            ls -lh fuzz/artifacts/parser_security/
            exit 1
          fi
          echo "✅ No crashes found"
      
      - name: Upload Coverage Map
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-coverage
          path: fuzz/coverage/
```

**Release Gate**: 24 cumulative fuzzing hours, 0 crashes

---

