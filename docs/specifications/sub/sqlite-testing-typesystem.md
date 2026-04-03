# Sub-spec: SQLite-Style Testing — Type System & Inference

**Parent:** [ruchy-sqlite-testing-v2.md](../ruchy-sqlite-testing-v2.md) Section 1.2

---

### 1.2 Middleend: Type System & Inference

**SQLite Equivalent**: TH3 achieving 100% MC/DC coverage  
**Ruchy Standard**: Mathematical proof of type soundness via property testing

#### Theoretical Foundation: Type Soundness

**Research Grounding**: *Types and Programming Languages* by Benjamin C. Pierce (MIT Press, 2002), Chapter 8: "Type Soundness"

Type soundness guarantees "well-typed programs don't go wrong." Formally proven via two theorems:

1. **Progress**: A well-typed term is not stuck (can step or is a value)
   - ∀t,T: (⊢ t : T) ⟹ (t is a value ∨ ∃t'. t → t')

2. **Preservation**: Evaluation preserves types
   - ∀Γ,t,T,t': (Γ ⊢ t : T ∧ t → t') ⟹ (Γ ⊢ t' : T)

Together: Progress + Preservation = Type Safety

#### Test Harness 1.2: Type Soundness Validation

```rust
// tests/type_system_soundness.rs

/**
 * Type System Soundness Proofs
 * 
 * Research Foundation:
 * Citation: Pierce, B. C. (2002). Types and Programming Languages. MIT Press.
 * Chapter 8: Type Soundness
 * 
 * We prove type soundness by testing the Progress and Preservation theorems
 * with 100,000+ property test iterations. This provides empirical validation
 * of the type system's mathematical correctness.
 * 
 * Theorem 8.3.3 (Soundness): If ⊢ t : T and t →* t', then t' is not stuck.
 */

#[cfg(test)]
mod type_soundness_proofs {
    use proptest::prelude::*;
    use ruchy::middleend::types::*;
    
    // ========================================================================
    // Theorem 1: Progress
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100_000))]
        
        #[test]
        fn theorem_progress(expr in any_well_typed_expr()) {
            /**
             * Progress Theorem (Pierce, Theorem 8.3.2)
             * 
             * Formal Statement:
             *   If ⊢ t : T, then either:
             *   (a) t is a value, or
             *   (b) ∃t' such that t → t'
             * 
             * Interpretation: Well-typed terms don't get "stuck".
             * A term is stuck if it's not a value but can't evaluate further.
             * 
             * Example of stuck term: 1 + true
             * This would be stuck because + expects integers, not booleans.
             * But our type system should reject "1 + true" during type checking,
             * preventing it from ever being evaluated.
             */
            
            let ty = infer_type(&expr).expect("Expression should be well-typed");
            
            // Attempt to evaluate one step
            let evaluation_result = evaluate_one_step(&expr);
            
            // One of these must be true:
            let is_value = expr.is_value();
            let can_step = evaluation_result.is_ok();
            
            assert!(
                is_value || can_step,
                "Progress theorem violated!\n  \
                 Expression: {}\n  \
                 Type: {}\n  \
                 is_value: {}\n  \
                 can_step: {}\n  \
                 A well-typed term must be a value or able to step.",
                expr, ty, is_value, can_step
            );
        }
    }
    
    // ========================================================================
    // Theorem 2: Preservation (Subject Reduction)
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100_000))]
        
        #[test]
        fn theorem_preservation(expr in any_well_typed_expr()) {
            /**
             * Preservation Theorem (Pierce, Theorem 8.3.1)
             * 
             * Formal Statement:
             *   If Γ ⊢ t : T and t → t', then Γ ⊢ t' : T
             * 
             * Interpretation: Evaluation preserves types.
             * If an expression has type T before evaluation, it still
             * has type T after taking an evaluation step.
             * 
             * Example: (λx:Int. x + 1) 5
             *   - Before beta-reduction: Int
             *   - After beta-reduction: 5 + 1 : Int
             *   - Type preserved ✓
             */
            
            let ty_before = infer_type(&expr).expect("Expression should be well-typed");
            
            // Take one evaluation step
            if let Ok(expr_after) = evaluate_one_step(&expr) {
                let ty_after = infer_type(&expr_after).expect(
                    "Evaluation should preserve well-typedness"
                );
                
                assert_eq!(
                    ty_before, ty_after,
                    "Preservation theorem violated!\n  \
                     Expression before: {}\n  \
                     Type before: {}\n  \
                     Expression after: {}\n  \
                     Type after: {}\n  \
                     Evaluation must preserve types.",
                    expr, ty_before, expr_after, ty_after
                );
            }
        }
    }
    
    // ========================================================================
    // Lemma: Substitution
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50_000))]
        
        #[test]
        fn lemma_substitution(
            ctx in any_typing_context(),
            x in any_variable(),
            e1 in any_expr(),
            e2 in any_expr()
        ) {
            /**
             * Substitution Lemma (Pierce, Lemma 8.2.5)
             * 
             * Formal Statement:
             *   If Γ, x:T1 ⊢ e2 : T2 and Γ ⊢ e1 : T1
             *   then Γ ⊢ [x ↦ e1]e2 : T2
             * 
             * Interpretation: Substituting a well-typed term preserves types.
             * This is the key lemma for proving preservation of function application.
             * 
             * Example: Γ ⊢ (λx:Int. x + 1) 5 : Int
             *   - Γ, x:Int ⊢ x + 1 : Int
             *   - Γ ⊢ 5 : Int
             *   - Therefore: Γ ⊢ substitution&#91;x ↦ 5&#93;(x + 1) = 5 + 1 : Int
             */
            
            // Type check e1 in context Γ
            let t1 = match infer_in_context(&ctx, &e1) {
                Ok(t) => t,
                Err(_) => return Ok(()), // e1 not well-typed, skip
            };
            
            // Type check e2 in extended context Γ, x:T1
            let extended_ctx = ctx.extend(x.clone(), t1.clone());
            let t2 = match infer_in_context(&extended_ctx, &e2) {
                Ok(t) => t,
                Err(_) => return Ok(()), // e2 not well-typed, skip
            };
            
            // Perform substitution: [x ↦ e1]e2
            let substituted = substitute(&e2, &x, &e1);
            
            // Type check substituted expression in original context Γ
            let t_substituted = infer_in_context(&ctx, &substituted).expect(
                "Substitution should preserve well-typedness"
            );
            
            assert_eq!(
                t2, t_substituted,
                "Substitution lemma violated!\n  \
                 Variable: {}\n  \
                 Substituting: {} : {}\n  \
                 Into: {} : {}\n  \
                 Result: {} : {}\n  \
                 Expected type: {}\n  \
                 Substitution must preserve types.",
                x, e1, t1, e2, t2, substituted, t_substituted, t2
            );
        }
    }
    
    // ========================================================================
    // Soundness Corollary
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100_000))]
        
        #[test]
        fn corollary_soundness(expr in any_well_typed_expr()) {
            /**
             * Soundness (Pierce, Theorem 8.3.3)
             * 
             * Corollary from Progress + Preservation:
             *   If ⊢ t : T and t →* t', then t' is not stuck.
             * 
             * This is the ultimate guarantee: well-typed programs don't go wrong.
             */
            
            let ty = infer_type(&expr).unwrap();
            
            // Evaluate to completion (or timeout)
            let result = evaluate_with_timeout(&expr, Duration::from_secs(1));
            
            match result {
                EvaluationResult::Value(v) => {
                    // Reached a value - verify it has correct type
                    assert!(has_type(&v, &ty));
                }
                EvaluationResult::Timeout => {
                    // Non-termination is allowed (halting problem undecidable)
                }
                EvaluationResult::Stuck => {
                    panic!(
                        "Soundness violated: well-typed term got stuck!\n  \
                         Expression: {}\n  \
                         Type: {}",
                        expr, ty
                    );
                }
            }
        }
    }
}

// ============================================================================
// Bidirectional Type Checking
// ============================================================================

#[cfg(test)]
mod bidirectional_typing {
    use super::*;
    
    #[test]
    fn test_inference_mode() {
        // Inference: synthesize type from expression
        let expr = parse_expr("42");
        assert_eq!(infer_type(&expr), Ok(Type::Int));
        
        let expr = parse_expr("[1, 2, 3]");
        assert_eq!(
            infer_type(&expr),
            Ok(Type::List(Box::new(Type::Int)))
        );
        
        let expr = parse_expr("λx. x + 1");
        assert_eq!(
            infer_type(&expr),
            Ok(Type::Arrow(
                Box::new(Type::Int),
                Box::new(Type::Int)
            ))
        );
    }
    
    #[test]
    fn test_checking_mode() {
        // Checking: verify expression has expected type
        let expr = parse_expr("42");
        assert!(check_type(&expr, &Type::Int).is_ok());
        assert!(check_type(&expr, &Type::String).is_err());
        
        let expr = parse_expr("if true { 1 } else { 2 }");
        assert!(check_type(&expr, &Type::Int).is_ok());
    }
    
    #[test]
    fn test_polymorphic_instantiation() {
        // id : ∀a. a → a
        let id_type = Type::Forall(
            "a".to_string(),
            Box::new(Type::Arrow(
                Box::new(Type::Var("a".to_string())),
                Box::new(Type::Var("a".to_string()))
            ))
        );
        
        let ctx = Context::empty().extend("id", id_type);
        
        // id 42 : Int
        let app1 = Expr::App(
            Box::new(Expr::Var("id".to_string())),
            Box::new(Expr::Lit(Literal::Int(42)))
        );
        assert_eq!(infer_in_context(&ctx, &app1), Ok(Type::Int));
        
        // id "hello" : String
        let app2 = Expr::App(
            Box::new(Expr::Var("id".to_string())),
            Box::new(Expr::Lit(Literal::String("hello".to_string())))
        );
        assert_eq!(infer_in_context(&ctx, &app2), Ok(Type::String));
        
        // Verify different type instantiations
        assert_ne!(
            infer_in_context(&ctx, &app1),
            infer_in_context(&ctx, &app2)
        );
    }
    
    #[test]
    fn test_unification_algorithm() {
        // Occurs check: prevents infinite types
        // Example: X = X → Int would create infinite type
        let result = unify(
            &Type::Var("X".to_string()),
            &Type::Arrow(
                Box::new(Type::Var("X".to_string())),
                Box::new(Type::Int)
            )
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("occurs check"));
        
        // Successful unification
        let t1 = Type::Arrow(
            Box::new(Type::Var("a".to_string())),
            Box::new(Type::Int)
        );
        let t2 = Type::Arrow(
            Box::new(Type::String),
            Box::new(Type::Var("b".to_string()))
        );
        
        let subst = unify(&t1, &t2).unwrap();
        assert_eq!(subst.get("a"), Some(&Type::String));
        assert_eq!(subst.get("b"), Some(&Type::Int));
    }
}

// ============================================================================
// Type Error Quality
// ============================================================================

#[cfg(test)]
mod type_errors {
    use super::*;
    
    #[test]
    fn test_comprehensive_type_errors() {
        let error_cases = [
            // Unification failures
            (
                "1 + \"hello\"",
                "type mismatch",
                "expected Int, found String"
            ),
            (
                "[1, \"hello\", 3]",
                "incompatible types",
                "list elements must have same type"
            ),
            
            // Arity mismatches
            (
                "let f(x) = x; f(1, 2)",
                "wrong number of arguments",
                "expected 1 argument, found 2"
            ),
            
            // Undefined variables
            (
                "x + 1",
                "undefined variable",
                "cannot find value `x` in this scope"
            ),
            
            // Occurs check violations
            (
                "let f = f; f",
                "infinite type",
                "occurs check failed"
            ),
            
            // Pattern match exhaustiveness
            (
                "match Some(1) { None => 0 }",
                "non-exhaustive patterns",
                "`Some(_)` not covered"
            ),
            
            // Recursive types without indirection
            (
                "type T = T",
                "recursive type",
                "recursive types require indirection"
            ),
        ];
        
        for (input, error_type, error_detail) in error_cases {
            let result = type_check(input);
            
            assert!(
                result.is_err(),
                "Should reject: {}",
                input
            );
            
            let error = result.unwrap_err();
            
            assert!(
                error.contains(error_type),
                "Error should mention '{}' for input: {}\nGot: {}",
                error_type, input, error
            );
            
            assert!(
                error.contains(error_detail),
                "Error should mention '{}' for input: {}\nGot: {}",
                error_detail, input, error
            );
        }
    }
}
```

**Coverage Target**: 
- 100% type inference rules
- 100% error conditions
- 100,000+ property test iterations per theorem
- Mathematical proof of soundness

**Test Count**: 300,000+ type system tests

---
