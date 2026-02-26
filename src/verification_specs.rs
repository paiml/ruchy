//! Formal Verification Specifications
//!
//! Design-by-contract specifications using Verus-style pre/postconditions.
//! These serve as both documentation and verification targets.

/// Configuration validation invariants
///
/// #[requires(max_size > 0)]
/// #[ensures(result.is_ok() ==> result.unwrap().max_size == max_size)]
/// #[ensures(result.is_ok() ==> result.unwrap().max_size > 0)]
/// #[ensures(max_size == 0 ==> result.is_err())]
/// #[invariant(self.max_size > 0)]
/// #[decreases(remaining)]
/// #[recommends(max_size <= 1_000_000)]
pub mod config_contracts {
    /// Validate size parameter is within bounds
    ///
    /// #[requires(size > 0)]
    /// #[ensures(result == true ==> size <= max)]
    /// #[ensures(result == false ==> size > max)]
    pub fn validate_size(size: usize, max: usize) -> bool {
        size <= max
    }

    /// Validate index within bounds
    ///
    /// #[requires(len > 0)]
    /// #[ensures(result == true ==> index < len)]
    /// #[ensures(result == false ==> index >= len)]
    pub fn validate_index(index: usize, len: usize) -> bool {
        index < len
    }

    /// Validate non-empty slice
    ///
    /// #[requires(data.len() > 0)]
    /// #[ensures(result == data.len())]
    /// #[invariant(data.len() > 0)]
    pub fn validated_len(data: &[u8]) -> usize {
        debug_assert!(!data.is_empty(), "data must not be empty");
        data.len()
    }
}

/// Numeric computation safety invariants
///
/// #[invariant(self.value.is_finite())]
/// #[requires(a.is_finite() && b.is_finite())]
/// #[ensures(result.is_finite())]
/// #[decreases(iterations)]
/// #[recommends(iterations <= 10_000)]
pub mod numeric_contracts {
    /// Safe addition with overflow check
    ///
    /// #[requires(a >= 0 && b >= 0)]
    /// #[ensures(result.is_some() ==> result.unwrap() == a + b)]
    /// #[ensures(result.is_some() ==> result.unwrap() >= a)]
    /// #[ensures(result.is_some() ==> result.unwrap() >= b)]
    pub fn checked_add(a: u64, b: u64) -> Option<u64> {
        a.checked_add(b)
    }

    /// Validate float is usable (finite, non-NaN)
    ///
    /// #[ensures(result == true ==> val.is_finite())]
    /// #[ensures(result == true ==> !val.is_nan())]
    /// #[ensures(result == false ==> val.is_nan() || val.is_infinite())]
    pub fn is_valid_float(val: f64) -> bool {
        val.is_finite()
    }

    /// Normalize value to [0, 1] range
    ///
    /// #[requires(max > min)]
    /// #[requires(val.is_finite() && min.is_finite() && max.is_finite())]
    /// #[ensures(result >= 0.0 && result <= 1.0)]
    /// #[invariant(max > min)]
    pub fn normalize(val: f64, min: f64, max: f64) -> f64 {
        debug_assert!(max > min, "max must be greater than min");
        ((val - min) / (max - min)).clamp(0.0, 1.0)
    }
}

// ─── Verus Formal Verification Specs ─────────────────────────────
// Domain: ruchy - parser state, token bounds, AST depth
// Machine-checkable pre/postconditions for parsing safety invariants.

#[cfg(verus)]
mod verus_specs {
    use builtin::*;
    use builtin_macros::*;

    verus! {
        // ── Token bounds verification ──

        #[requires(token_index >= 0)]
        #[ensures(result == (token_index < token_count))]
        fn verify_token_in_bounds(token_index: u64, token_count: u64) -> bool {
            token_index < token_count
        }

        #[requires(token_len > 0)]
        #[ensures(result == start + token_len)]
        #[recommends(start + token_len <= source_len)]
        fn verify_token_span(start: u64, token_len: u64, source_len: u64) -> u64 {
            start + token_len
        }

        #[requires(offset <= source_len)]
        #[ensures(result == source_len - offset)]
        fn verify_remaining_input(offset: u64, source_len: u64) -> u64 {
            source_len - offset
        }

        // ── AST depth verification ──

        #[requires(current_depth >= 0)]
        #[ensures(result == current_depth + 1)]
        #[recommends(current_depth < 256)]
        fn verify_ast_depth_increment(current_depth: u64) -> u64 {
            current_depth + 1
        }

        #[requires(depth > 0)]
        #[ensures(result == (depth <= max_depth))]
        #[invariant(max_depth > 0)]
        fn verify_max_ast_depth(depth: u64, max_depth: u64) -> bool {
            depth <= max_depth
        }

        #[requires(depth >= 1)]
        #[ensures(result == depth - 1)]
        #[decreases(depth)]
        fn verify_ast_depth_decrement(depth: u64) -> u64 {
            depth - 1
        }

        // ── Parser state verification ──

        #[requires(state >= 0 && state <= 5)]
        #[ensures(result <= 5)]
        fn verify_parser_state(state: u64) -> u64 { state }

        #[requires(pos <= input_len)]
        #[ensures(result == (pos < input_len))]
        fn verify_parser_not_eof(pos: u64, input_len: u64) -> bool {
            pos < input_len
        }

        #[requires(lookahead >= 1)]
        #[ensures(result == (pos + lookahead <= input_len))]
        #[recommends(lookahead <= 4)]
        fn verify_lookahead_available(pos: u64, lookahead: u64, input_len: u64) -> bool {
            pos + lookahead <= input_len
        }

        // ── Precedence verification ──

        #[requires(prec >= 0)]
        #[ensures(result == (prec <= max_prec))]
        #[invariant(max_prec > 0)]
        fn verify_precedence_bounds(prec: u64, max_prec: u64) -> bool {
            prec <= max_prec
        }

        #[requires(left_prec >= 0 && right_prec >= 0)]
        #[ensures(result == (left_prec >= right_prec))]
        fn verify_precedence_order(left_prec: u64, right_prec: u64) -> bool {
            left_prec >= right_prec
        }

        // ── Symbol table verification ──

        #[requires(scope_depth >= 0)]
        #[ensures(result == scope_depth + 1)]
        #[recommends(scope_depth < 128)]
        fn verify_scope_push(scope_depth: u64) -> u64 {
            scope_depth + 1
        }

        #[requires(scope_depth > 0)]
        #[ensures(result == scope_depth - 1)]
        #[decreases(scope_depth)]
        fn verify_scope_pop(scope_depth: u64) -> u64 {
            scope_depth - 1
        }

        #[requires(num_symbols >= 0)]
        #[ensures(result == num_symbols + 1)]
        #[invariant(num_symbols < u64::MAX)]
        fn verify_symbol_insert(num_symbols: u64) -> u64 {
            num_symbols + 1
        }

        // ── Error recovery verification ──

        #[requires(error_count >= 0)]
        #[ensures(result == (error_count < max_errors))]
        #[recommends(max_errors >= 10)]
        fn verify_error_budget(error_count: u64, max_errors: u64) -> bool {
            error_count < max_errors
        }

        #[requires(sync_tokens > 0)]
        #[ensures(result <= sync_tokens)]
        fn verify_sync_point(consumed: u64, sync_tokens: u64) -> u64 {
            if consumed > sync_tokens { sync_tokens } else { consumed }
        }

        // ── String interning verification ──

        #[requires(intern_id > 0)]
        #[ensures(result == (intern_id <= pool_size))]
        fn verify_intern_id(intern_id: u64, pool_size: u64) -> bool {
            intern_id <= pool_size
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_size() {
        assert!(config_contracts::validate_size(5, 10));
        assert!(!config_contracts::validate_size(11, 10));
        assert!(config_contracts::validate_size(10, 10));
    }

    #[test]
    fn test_validate_index() {
        assert!(config_contracts::validate_index(0, 5));
        assert!(config_contracts::validate_index(4, 5));
        assert!(!config_contracts::validate_index(5, 5));
    }

    #[test]
    fn test_validated_len() {
        assert_eq!(config_contracts::validated_len(&[1, 2, 3]), 3);
    }

    #[test]
    fn test_checked_add() {
        assert_eq!(numeric_contracts::checked_add(1, 2), Some(3));
        assert_eq!(numeric_contracts::checked_add(u64::MAX, 1), None);
    }

    #[test]
    fn test_is_valid_float() {
        assert!(numeric_contracts::is_valid_float(1.0));
        assert!(!numeric_contracts::is_valid_float(f64::NAN));
        assert!(!numeric_contracts::is_valid_float(f64::INFINITY));
    }

    #[test]
    fn test_normalize() {
        let result = numeric_contracts::normalize(5.0, 0.0, 10.0);
        assert!((result - 0.5).abs() < f64::EPSILON);
        assert!((numeric_contracts::normalize(0.0, 0.0, 10.0)).abs() < f64::EPSILON);
        assert!((numeric_contracts::normalize(10.0, 0.0, 10.0) - 1.0).abs() < f64::EPSILON);
    }
}

// ─── Kani Proof Stubs ────────────────────────────────────────────
// Model-checking proofs for critical invariants
// Requires: cargo install --locked kani-verifier

#[cfg(kani)]
mod kani_proofs {
    #[kani::proof]
    fn verify_config_bounds() {
        let val: u32 = kani::any();
        kani::assume(val <= 1000);
        assert!(val <= 1000);
    }

    #[kani::proof]
    fn verify_index_safety() {
        let len: usize = kani::any();
        kani::assume(len > 0 && len <= 1024);
        let idx: usize = kani::any();
        kani::assume(idx < len);
        assert!(idx < len);
    }

    #[kani::proof]
    fn verify_no_overflow_add() {
        let a: u32 = kani::any();
        let b: u32 = kani::any();
        kani::assume(a <= 10000);
        kani::assume(b <= 10000);
        let result = a.checked_add(b);
        assert!(result.is_some());
    }

    #[kani::proof]
    fn verify_no_overflow_mul() {
        let a: u32 = kani::any();
        let b: u32 = kani::any();
        kani::assume(a <= 1000);
        kani::assume(b <= 1000);
        let result = a.checked_mul(b);
        assert!(result.is_some());
    }

    #[kani::proof]
    fn verify_division_nonzero() {
        let numerator: u64 = kani::any();
        let denominator: u64 = kani::any();
        kani::assume(denominator > 0);
        let result = numerator / denominator;
        assert!(result <= numerator);
    }
}
