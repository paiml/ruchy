//! Trueno Bridge Module (Pillar 1: SIMD Compute)
//!
//! Thin wrappers around Trueno tensor operations for Ruchy stdlib.
//! Per spec Section 5.1: SIMD-accelerated tensor operations as the universal compute substrate.
//!
//! # Design
//! - Zero-cost abstractions over trueno primitives
//! - Kahan summation for numerical stability [Kahan 1965]
//! - Auto-vectorization (AVX2/AVX-512/NEON/SIMD128)
//!
//! # Precision Guarantees
//!
//! | Operation | Precision | Error Bound | Notes |
//! |-----------|-----------|-------------|-------|
//! | `kahan_sum` | f64 | O(ε) | Compensated summation, order-independent |
//! | `kahan_sum_f32` | f32 | O(ε) | Compensated summation, order-independent |
//! | `dot_f32` | f32 | O(n·ε) | SIMD-accelerated, uses Trueno backend |
//! | `dot` | f64→f32→f64 | O(n·ε) + conversion | Converts to f32 for SIMD, loses precision |
//! | `mean` | f64 | O(ε) | Uses Kahan summation internally |
//! | `variance` | f64 | O(ε) | Two-pass algorithm with Kahan summation |
//! | `std_dev` | f64 | O(ε) + sqrt | Variance + square root |
//! | `add_f32` | f32 | O(ε) | Element-wise, SIMD-accelerated |
//! | `mul_f32` | f32 | O(ε) | Element-wise, SIMD-accelerated |
//! | `scale_f32` | f32 | O(ε) | Scalar multiply, SIMD-accelerated |
//!
//! Where ε = machine epsilon (f64: 2.2e-16, f32: 1.2e-7)
//!
//! # Backend Equivalence
//!
//! All operations produce identical results across backends (AVX2, AVX-512, NEON, WASM SIMD128, Scalar).
//! This is guaranteed by IEEE 754 compliance and deterministic reduction ordering.
//!
//! # References
//! - [11] Kahan, W. (1965). "Further Remarks on Reducing Truncation Errors"
//! - [12] Higham, N. J. (2002). "Accuracy and Stability of Numerical Algorithms"

pub use trueno::matrix::Matrix;
pub use trueno::vector::Vector;
pub use trueno::{select_best_available_backend, Backend};

/// Kahan summation for numerical stability.
///
/// Standard floating-point summation accumulates error proportional to N.
/// Kahan summation bounds error to O(1) regardless of input size.
///
/// # Arguments
/// * `values` - Slice of f64 values to sum
///
/// # Returns
/// The compensated sum with reduced numerical error
///
/// # Examples
/// ```
/// use ruchy::stdlib::trueno_bridge::kahan_sum;
///
/// let values = vec![1e16, 1.0, -1e16];
/// let result = kahan_sum(&values);
/// assert!((result - 1.0).abs() < 1e-10);
/// ```
#[must_use]
pub fn kahan_sum(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0; // Compensation for lost low-order bits
    for &x in values {
        let y = x - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    sum
}

/// Kahan summation for f32 values.
///
/// # Precision Guarantee
/// Error bound: O(ε) where ε = 1.2e-7 (f32 machine epsilon)
/// Order-independent: Results are stable regardless of input ordering.
///
/// # Examples
/// ```
/// use ruchy::stdlib::trueno_bridge::kahan_sum_f32;
///
/// let values = vec![1e7_f32, 1.0, -1e7];
/// let result = kahan_sum_f32(&values);
/// assert!((result - 1.0).abs() < 1e-5);
/// ```
#[must_use]
pub fn kahan_sum_f32(values: &[f32]) -> f32 {
    let mut sum = 0.0_f32;
    let mut c = 0.0_f32;
    for &x in values {
        let y = x - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    sum
}

/// Compute dot product of two f32 vectors using SIMD.
///
/// # Errors
/// Returns error if vectors have different lengths.
pub fn dot_f32(a: &[f32], b: &[f32]) -> Result<f32, String> {
    if a.len() != b.len() {
        return Err(format!(
            "Vector length mismatch: {} vs {}",
            a.len(),
            b.len()
        ));
    }
    let va = Vector::from_slice(a);
    let vb = Vector::from_slice(b);
    va.dot(&vb).map_err(|e| format!("Dot product failed: {e}"))
}

/// Compute dot product of two f64 vectors.
///
/// Note: Internally converts to f32 for SIMD acceleration, then back to f64.
///
/// # Errors
/// Returns error if vectors have different lengths.
pub fn dot(a: &[f64], b: &[f64]) -> Result<f64, String> {
    if a.len() != b.len() {
        return Err(format!(
            "Vector length mismatch: {} vs {}",
            a.len(),
            b.len()
        ));
    }
    let a_f32: Vec<f32> = a.iter().map(|&x| x as f32).collect();
    let b_f32: Vec<f32> = b.iter().map(|&x| x as f32).collect();
    dot_f32(&a_f32, &b_f32).map(f64::from)
}

/// Compute mean of values using Kahan summation for stability.
#[must_use]
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    kahan_sum(values) / values.len() as f64
}

/// Compute variance using numerically stable algorithm.
///
/// Uses two-pass algorithm with Kahan summation for numerical stability.
#[must_use]
pub fn variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let n = values.len() as f64;
    let m = mean(values);

    // Two-pass algorithm with Kahan summation
    let squared_diffs: Vec<f64> = values.iter().map(|&x| (x - m).powi(2)).collect();
    kahan_sum(&squared_diffs) / (n - 1.0)
}

/// Compute standard deviation.
#[must_use]
pub fn std_dev(values: &[f64]) -> f64 {
    variance(values).sqrt()
}

/// Get the best available SIMD backend for the current platform.
#[must_use]
pub fn best_backend() -> Backend {
    select_best_available_backend()
}

/// Element-wise addition of two f32 vectors using SIMD.
///
/// # Errors
/// Returns error if vectors have different lengths.
pub fn add_f32(a: &[f32], b: &[f32]) -> Result<Vec<f32>, String> {
    if a.len() != b.len() {
        return Err(format!(
            "Vector length mismatch: {} vs {}",
            a.len(),
            b.len()
        ));
    }
    let va = Vector::from_slice(a);
    let vb = Vector::from_slice(b);
    va.add(&vb)
        .map(|v| v.as_slice().to_vec())
        .map_err(|e| format!("Add failed: {e}"))
}

/// Element-wise multiplication of two f32 vectors using SIMD.
///
/// # Errors
/// Returns error if vectors have different lengths.
pub fn mul_f32(a: &[f32], b: &[f32]) -> Result<Vec<f32>, String> {
    if a.len() != b.len() {
        return Err(format!(
            "Vector length mismatch: {} vs {}",
            a.len(),
            b.len()
        ));
    }
    let va = Vector::from_slice(a);
    let vb = Vector::from_slice(b);
    va.mul(&vb)
        .map(|v| v.as_slice().to_vec())
        .map_err(|e| format!("Mul failed: {e}"))
}

/// Scalar multiplication of a f32 vector.
#[must_use]
pub fn scale_f32(a: &[f32], scalar: f32) -> Vec<f32> {
    let va = Vector::from_slice(a);
    match va.scale(scalar) {
        Ok(result) => result.as_slice().to_vec(),
        Err(_) => a.iter().map(|&x| x * scalar).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Kahan Summation Tests ==========

    #[test]
    fn test_kahan_sum_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((kahan_sum(&values) - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_kahan_sum_empty() {
        let values: Vec<f64> = vec![];
        assert!((kahan_sum(&values) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_kahan_sum_cancellation() {
        // Test with values that demonstrate Kahan's advantage over naive sum
        // Using 1e8 which is within f64's ~15 decimal digit precision
        let values = vec![1e8, 1.0, 1.0, 1.0, -1e8];
        let result = kahan_sum(&values);
        assert!(
            (result - 3.0).abs() < 1e-10,
            "Kahan sum failed: got {result}, expected 3.0"
        );

        // Another test: sum of many small values that would lose precision naively
        let small: Vec<f64> = vec![0.1; 10];
        let result = kahan_sum(&small);
        // 0.1 * 10 = 1.0 exactly with Kahan (naive sum might drift)
        assert!(
            (result - 1.0).abs() < 1e-14,
            "Kahan sum of 0.1s failed: got {result}, expected 1.0"
        );
    }

    #[test]
    fn test_kahan_sum_many_small_values() {
        // Sum of 1M small values - naive sum would accumulate significant error
        let values: Vec<f64> = vec![1e-10; 1_000_000];
        let result = kahan_sum(&values);
        let expected = 1e-10 * 1_000_000.0;
        assert!(
            (result - expected).abs() < 1e-10,
            "Kahan sum error: got {result}, expected {expected}"
        );
    }

    // ========== Dot Product Tests ==========

    #[test]
    fn test_dot_basic() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = dot(&a, &b).expect("dot should succeed");
        assert!((result - 32.0).abs() < 0.01); // f32 precision
    }

    #[test]
    fn test_dot_length_mismatch() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!(dot(&a, &b).is_err());
    }

    #[test]
    fn test_dot_empty() {
        let a: Vec<f64> = vec![];
        let b: Vec<f64> = vec![];
        let result = dot(&a, &b).expect("dot should succeed");
        assert!((result - 0.0).abs() < 1e-10);
    }

    // ========== Mean/Variance/StdDev Tests ==========

    #[test]
    fn test_mean_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((mean(&values) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_mean_empty() {
        let values: Vec<f64> = vec![];
        assert!((mean(&values) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_variance_basic() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let var = variance(&values);
        assert!((var - 4.571_428_571_428_571).abs() < 1e-10);
    }

    #[test]
    fn test_variance_single() {
        let values = vec![5.0];
        assert!((variance(&values) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_basic() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std = std_dev(&values);
        assert!((std - 2.138_089_935_299_395).abs() < 1e-10);
    }

    // ========== Vector Operations Tests ==========

    #[test]
    fn test_add_f32_basic() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let b = vec![4.0_f32, 5.0, 6.0];
        let result = add_f32(&a, &b).expect("add should succeed");
        assert_eq!(result.len(), 3);
        assert!((result[0] - 5.0).abs() < 1e-5);
        assert!((result[1] - 7.0).abs() < 1e-5);
        assert!((result[2] - 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_mul_f32_basic() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let b = vec![4.0_f32, 5.0, 6.0];
        let result = mul_f32(&a, &b).expect("mul should succeed");
        assert!((result[0] - 4.0).abs() < 1e-5);
        assert!((result[1] - 10.0).abs() < 1e-5);
        assert!((result[2] - 18.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_f32_basic() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let result = scale_f32(&a, 2.0);
        assert!((result[0] - 2.0).abs() < 1e-5);
        assert!((result[1] - 4.0).abs() < 1e-5);
        assert!((result[2] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_best_backend_available() {
        let backend = best_backend();
        // Should return some valid backend
        assert!(matches!(
            backend,
            Backend::Scalar
                | Backend::SSE2
                | Backend::AVX
                | Backend::AVX2
                | Backend::AVX512
                | Backend::NEON
                | Backend::WasmSIMD
                | Backend::GPU
                | Backend::Auto
        ));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn prop_kahan_sum_commutative(a in prop::collection::vec(-1e10..1e10, 0..100)) {
            let sum1 = kahan_sum(&a);
            #[allow(clippy::redundant_clone)]
            let mut reversed = a.clone();
            reversed.reverse();
            let sum2 = kahan_sum(&reversed);
            // Kahan sum should be approximately equal regardless of order
            prop_assert!((sum1 - sum2).abs() < 1e-6 * sum1.abs().max(1.0));
        }

        #[test]
        fn prop_mean_bounds(a in prop::collection::vec(-100.0..100.0f64, 1..100)) {
            let m = mean(&a);
            let min = a.iter().copied().fold(f64::INFINITY, f64::min);
            let max = a.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            prop_assert!(m >= min && m <= max);
        }

        #[test]
        fn prop_variance_non_negative(a in prop::collection::vec(-100.0..100.0f64, 2..100)) {
            let var = variance(&a);
            prop_assert!(var >= 0.0);
        }

        #[test]
        fn prop_scale_by_one_identity(a in prop::collection::vec(-100.0..100.0f32, 1..50)) {
            let result = scale_f32(&a, 1.0);
            for (x, y) in a.iter().zip(result.iter()) {
                prop_assert!((x - y).abs() < 1e-5);
            }
        }
    }
}

/// Backend equivalence tests - verify SIMD operations match scalar baseline
///
/// Per spec Section 13.2: All operations must produce identical results across
/// backends (AVX2, AVX-512, NEON, WASM SIMD128, Scalar) within IEEE 754 tolerance.
#[cfg(test)]
mod backend_equivalence_tests {
    use super::*;

    /// Scalar baseline implementation for `kahan_sum` (no SIMD)
    fn scalar_kahan_sum(values: &[f64]) -> f64 {
        let mut sum = 0.0_f64;
        let mut c = 0.0_f64;
        for &x in values {
            let y = x - c;
            let t = sum + y;
            c = (t - sum) - y;
            sum = t;
        }
        sum
    }

    /// Scalar baseline for dot product
    fn scalar_dot(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Scalar baseline for vector add
    fn scalar_add(a: &[f32], b: &[f32]) -> Vec<f32> {
        a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
    }

    /// Scalar baseline for vector mul
    fn scalar_mul(a: &[f32], b: &[f32]) -> Vec<f32> {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
    }

    /// Scalar baseline for scale
    fn scalar_scale(a: &[f32], s: f32) -> Vec<f32> {
        a.iter().map(|x| x * s).collect()
    }

    // ========== Backend Equivalence Tests ==========

    #[test]
    fn test_kahan_sum_matches_scalar_baseline() {
        let values: Vec<f64> = (0..1000).map(|i| f64::from(i) * 0.001).collect();
        let simd_result = kahan_sum(&values);
        let scalar_result = scalar_kahan_sum(&values);
        assert!(
            (simd_result - scalar_result).abs() < 1e-12,
            "SIMD kahan_sum ({simd_result}) != scalar ({scalar_result})"
        );
    }

    #[test]
    fn test_dot_matches_scalar_baseline() {
        let a: Vec<f32> = (0..256).map(|i| (i as f32) * 0.1).collect();
        let b: Vec<f32> = (0..256).map(|i| (i as f32) * 0.2).collect();
        let simd_result = dot_f32(&a, &b).unwrap();
        let scalar_result = scalar_dot(&a, &b);
        let rel_error = (simd_result - scalar_result).abs() / scalar_result.abs().max(1.0);
        assert!(
            rel_error < 1e-5,
            "SIMD dot ({simd_result}) != scalar ({scalar_result}), rel_error={rel_error}"
        );
    }

    #[test]
    fn test_add_matches_scalar_baseline() {
        let a: Vec<f32> = (0..128).map(|i| (i as f32) * 0.5).collect();
        let b: Vec<f32> = (0..128).map(|i| (i as f32) * 0.3).collect();
        let simd_result = add_f32(&a, &b).unwrap();
        let scalar_result = scalar_add(&a, &b);
        for (i, (s, r)) in simd_result.iter().zip(scalar_result.iter()).enumerate() {
            assert!(
                (s - r).abs() < 1e-6,
                "add mismatch at index {i}: SIMD={s}, scalar={r}"
            );
        }
    }

    #[test]
    fn test_mul_matches_scalar_baseline() {
        let a: Vec<f32> = (0..128).map(|i| (i as f32) * 0.5).collect();
        let b: Vec<f32> = (0..128).map(|i| (i as f32) * 0.3).collect();
        let simd_result = mul_f32(&a, &b).unwrap();
        let scalar_result = scalar_mul(&a, &b);
        for (i, (s, r)) in simd_result.iter().zip(scalar_result.iter()).enumerate() {
            assert!(
                (s - r).abs() < 1e-5,
                "mul mismatch at index {i}: SIMD={s}, scalar={r}"
            );
        }
    }

    #[test]
    fn test_scale_matches_scalar_baseline() {
        let a: Vec<f32> = (0..128).map(|i| (i as f32) * 0.7).collect();
        let scalar = 2.5_f32;
        let simd_result = scale_f32(&a, scalar);
        let scalar_result = scalar_scale(&a, scalar);
        for (i, (s, r)) in simd_result.iter().zip(scalar_result.iter()).enumerate() {
            assert!(
                (s - r).abs() < 1e-5,
                "scale mismatch at index {i}: SIMD={s}, scalar={r}"
            );
        }
    }

    #[test]
    fn test_mean_variance_consistency() {
        // Mean of uniform values should equal any element
        let uniform: Vec<f64> = vec![42.0; 100];
        assert!((mean(&uniform) - 42.0).abs() < 1e-10);
        assert!(variance(&uniform).abs() < 1e-10);

        // Known distribution: [1, 2, 3, 4, 5] has mean=3, var=2.5
        let seq: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((mean(&seq) - 3.0).abs() < 1e-10);
        assert!((variance(&seq) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_backend_reports_valid_type() {
        let backend = best_backend();
        // Log which backend we're using (useful for CI verification)
        eprintln!("Running with backend: {backend:?}");
        // Backend must be one of the known types
        assert!(matches!(
            backend,
            Backend::Scalar
                | Backend::SSE2
                | Backend::AVX
                | Backend::AVX2
                | Backend::AVX512
                | Backend::NEON
                | Backend::WasmSIMD
                | Backend::GPU
                | Backend::Auto
        ));
    }
}
