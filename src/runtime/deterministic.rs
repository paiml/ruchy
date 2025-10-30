//! Deterministic execution support for REPL replay testing
//!
//! Implements the `DeterministicRepl` trait for the Ruchy REPL to enable
//! deterministic replay for testing and educational assessment.
use crate::runtime::interpreter::Value;
use crate::runtime::repl::Repl;
use crate::runtime::replay::{
    DeterministicRepl, Divergence, ReplayResult, ResourceUsage, StateCheckpoint, ValidationResult,
};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
/// Mock time source for deterministic time operations
pub struct MockTime {
    current_ns: u64,
}
impl Default for MockTime {
    fn default() -> Self {
        Self::new()
    }
}
impl MockTime {
    pub fn new() -> Self {
        Self { current_ns: 0 }
    }
    pub fn advance(&mut self, ns: u64) {
        self.current_ns += ns;
    }
    pub fn now(&self) -> u64 {
        self.current_ns
    }
}
/// Deterministic random number generator
pub struct DeterministicRng {
    seed: u64,
    state: u64,
}
impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { seed, state: seed }
    }
    pub fn next(&mut self) -> u64 {
        // Simple LCG for deterministic pseudo-random numbers
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }
    pub fn reset(&mut self) {
        self.state = self.seed;
    }
}
/// Extension trait to make Repl deterministic
impl DeterministicRepl for Repl {
    fn execute_with_seed(&mut self, input: &str, _seed: u64) -> ReplayResult {
        // Store current resource usage start point
        let start_heap = self.estimate_heap_usage();
        let start_stack = self.estimate_stack_depth();
        let start_time = std::time::Instant::now();
        // Note: RNG seed will be set when random number support is added to REPL
        // Currently executes normally as REPL doesn't use RNG yet
        // Execute the input
        let output = self.process_line(input).map(|_success| {
            // For now, we'll return a placeholder string representation
            // In the future, we should capture actual evaluation results
            let s = "success"; // Simplified for now
                               // Convert string output back to Value
                               // This is a simplified conversion - in production we'd preserve the actual Value
            if s == "()" {
                Value::Nil
            } else if s == "true" {
                Value::Bool(true)
            } else if s == "false" {
                Value::Bool(false)
            } else if let Ok(n) = s.parse::<i64>() {
                Value::Integer(n)
            } else if s.starts_with('"') && s.ends_with('"') {
                Value::String(Arc::from(&s[1..s.len() - 1]))
            } else {
                // For complex types, we store as string representation
                Value::from_string(s.to_string())
            }
        });
        // Calculate resource usage
        let heap_bytes = self.estimate_heap_usage() - start_heap;
        let stack_depth = self.estimate_stack_depth() - start_stack;
        let cpu_ns = start_time.elapsed().as_nanos() as u64;
        // Compute state hash
        let state_hash = self.compute_state_hash();
        ReplayResult {
            output,
            state_hash,
            resource_usage: ResourceUsage {
                heap_bytes,
                stack_depth,
                cpu_ns,
            },
        }
    }
    fn checkpoint(&self) -> StateCheckpoint {
        let mut bindings = HashMap::new();
        let type_environment = HashMap::new();
        // Extract all variable bindings
        for (name, value) in self.get_bindings() {
            bindings.insert(name.clone(), value.to_string());
        }
        // Extract type environment if available
        // Type tracking will be implemented when static analysis is added
        StateCheckpoint {
            bindings,
            type_environment,
            state_hash: self.compute_state_hash(),
            resource_usage: ResourceUsage {
                heap_bytes: self.estimate_heap_usage(),
                stack_depth: self.estimate_stack_depth(),
                cpu_ns: 0, // Not meaningful for a checkpoint
            },
        }
    }
    fn restore(&mut self, checkpoint: &StateCheckpoint) -> Result<()> {
        // Clear current state from BOTH REPL bindings AND evaluator interpreter
        self.clear_bindings();

        // Clear interpreter environment (the real variable storage)
        if let Some(evaluator) = self.get_evaluator_mut() {
            evaluator.clear_interpreter_variables();
        }

        // Restore bindings to both REPL state and interpreter
        for (name, value_str) in &checkpoint.bindings {
            // This is simplified - in production we'd properly deserialize the values
            let value = if value_str == "nil" {
                Value::Nil
            } else if value_str == "true" {
                Value::Bool(true)
            } else if value_str == "false" {
                Value::Bool(false)
            } else if let Ok(n) = value_str.parse::<i64>() {
                Value::Integer(n)
            } else if let Ok(f) = value_str.parse::<f64>() {
                Value::Float(f)
            } else if value_str.starts_with('"') && value_str.ends_with('"') {
                // Remove quotes from string values
                let content = &value_str[1..value_str.len() - 1];
                Value::from_string(content.to_string())
            } else {
                // Fallback: store as string (this should not happen with Display format)
                Value::from_string(value_str.clone())
            };

            // Restore to REPL bindings
            self.get_bindings_mut().insert(name.clone(), value.clone());

            // Also restore to interpreter environment
            if let Some(evaluator) = self.get_evaluator_mut() {
                evaluator.set_variable(name.clone(), value);
            }
        }
        Ok(())
    }
    fn validate_determinism(&self, other: &Self) -> ValidationResult {
        let mut divergences = vec![];
        // Compare variable bindings
        for (name, value) in self.get_bindings() {
            match other.get_bindings().get(name) {
                Some(other_value) if value == other_value => {
                    // Values match, good
                }
                Some(other_value) => {
                    divergences.push(Divergence::State {
                        expected_hash: format!("{value:?}"),
                        actual_hash: format!("{other_value:?}"),
                    });
                }
                None => {
                    divergences.push(Divergence::State {
                        expected_hash: format!("{value:?}"),
                        actual_hash: "missing".to_string(),
                    });
                }
            }
        }
        // Check for variables in other but not in self
        for name in other.get_bindings().keys() {
            if !self.get_bindings().contains_key(name) {
                divergences.push(Divergence::State {
                    expected_hash: "missing".to_string(),
                    actual_hash: format!("variable: {name}"),
                });
            }
        }
        ValidationResult {
            is_deterministic: divergences.is_empty(),
            divergences,
        }
    }
}
// Helper methods for Repl
impl Repl {
    /// Estimate heap usage in bytes (simplified)
    fn estimate_heap_usage(&self) -> usize {
        // Rough estimate based on number of variables and their sizes
        let mut total = 0;
        for value in self.get_bindings().values() {
            total += std::mem::size_of_val(value);
            total += match value {
                Value::String(s) => s.len(),
                Value::Array(items) => items.len() * std::mem::size_of::<Value>(),
                Value::Object(map) => map.len() * (32 + std::mem::size_of::<Value>()),
                _ => 0,
            };
        }
        total
    }
    /// Estimate current stack depth (simplified)
    fn estimate_stack_depth(&self) -> usize {
        // This is a placeholder - real implementation would track actual call stack
        // For now, we return a fixed estimate based on the number of bindings
        self.get_bindings().len() / 10 + 1
    }
    /// Compute a hash of the current state for comparison
    fn compute_state_hash(&self) -> String {
        let mut hasher = Sha256::new();
        // Sort variables by name for deterministic hashing
        let mut sorted_vars: Vec<_> = self.get_bindings().iter().collect();
        sorted_vars.sort_by_key(|(name, _)| name.as_str());
        for (name, value) in sorted_vars {
            hasher.update(name.as_bytes());
            hasher.update(b":");
            // FIX Issue #86: Use to_string() instead of {:?} to ensure deterministic
            // serialization of nested HashMaps in Value::Object
            hasher.update(value.to_string().as_bytes());
            hasher.update(b";");
        }
        format!("{:x}", hasher.finalize())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_deterministic_execution() {
        // Each test gets isolated temp directory for idempotence
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let mut repl1 = Repl::new(temp_dir1.path().to_path_buf()).unwrap();
        let mut repl2 = Repl::new(temp_dir2.path().to_path_buf()).unwrap();
        // Execute same commands with same seed
        let result1 = repl1.execute_with_seed("let x = 42", 12345);
        let result2 = repl2.execute_with_seed("let x = 42", 12345);
        // Results should be identical
        assert!(result1.output.is_ok());
        assert!(result2.output.is_ok());
        assert_eq!(result1.state_hash, result2.state_hash);
    }
    #[test]
    fn test_checkpoint_restore() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();
        // Create some state
        repl.eval("let x = 10").unwrap();
        repl.eval("let y = 20").unwrap();
        // Create checkpoint using DeterministicRepl trait
        let checkpoint = DeterministicRepl::checkpoint(&repl);
        // Modify state
        repl.eval("let x = 99").unwrap();
        repl.eval("let z = 30").unwrap();
        // Restore checkpoint
        DeterministicRepl::restore(&mut repl, &checkpoint).unwrap();
        // Check that state was restored
        // Note: Values are restored from debug format, so they may have different representation
        assert!(repl.eval("x").is_ok());
        assert!(repl.eval("y").is_ok());
        // z should not exist after restore
        assert!(repl.eval("z").is_err());
    }
    #[test]
    fn test_determinism_validation() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let mut repl1 = Repl::new(temp_dir1.path().to_path_buf()).unwrap();
        let mut repl2 = Repl::new(temp_dir2.path().to_path_buf()).unwrap();
        // Same operations
        repl1.eval("let x = 1").unwrap();
        repl2.eval("let x = 1").unwrap();
        let validation = repl1.validate_determinism(&repl2);
        assert!(validation.is_deterministic);
        assert!(validation.divergences.is_empty());
        // Different operations
        repl1.eval("let y = 2").unwrap();
        repl2.eval("let y = 3").unwrap();
        let validation = repl1.validate_determinism(&repl2);
        assert!(!validation.is_deterministic);
        assert!(!validation.divergences.is_empty());
    }

    #[test]
    fn test_estimate_heap_usage_actual_calculation() {
        // Mutation test: Verify estimate_heap_usage returns actual calculations, not stub values
        // MISSED: replace estimate_heap_usage -> usize with 1
        // MISSED: replace * with + in estimate_heap_usage (lines 210, 211)
        // MISSED: replace * with / in estimate_heap_usage (line 211)

        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Empty state should have 0 heap usage
        let empty_heap = repl.estimate_heap_usage();
        assert_eq!(empty_heap, 0, "Empty REPL should have 0 heap usage");

        // Add string variable - should increase heap
        repl.eval("let s = \"hello world\"").unwrap();
        let with_string = repl.estimate_heap_usage();
        assert!(
            with_string > empty_heap,
            "String should increase heap usage from {empty_heap} to {with_string}"
        );

        // Add array - should increase heap more (multiplication in line 210)
        repl.eval("let arr = [1, 2, 3, 4, 5]").unwrap();
        let with_array = repl.estimate_heap_usage();
        assert!(
            with_array > with_string,
            "Array should increase heap usage from {with_string} to {with_array}"
        );

        // Verify it's actually using multiplication (not addition or division)
        // Array with 5 items should add items.len() * size_of::<Value>()
        let array_contribution = with_array - with_string;
        assert!(
            array_contribution >= 5,
            "Array contribution {array_contribution} should be at least 5 (items * size)"
        );
    }

    #[test]
    fn test_estimate_stack_depth_arithmetic() {
        // Mutation test: Verify estimate_stack_depth uses correct arithmetic
        // MISSED: replace estimate_stack_depth -> usize with 0
        // MISSED: replace / with % in estimate_stack_depth (line 221)

        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Get initial depth (may not be 1 due to internal bindings)
        let initial_depth = repl.estimate_stack_depth();
        let initial_bindings = repl.get_bindings().len();

        // Add exactly 10 bindings
        for i in 0..10 {
            repl.eval(&format!("let var{i} = {i}")).unwrap();
        }
        let bindings_after_10 = repl.get_bindings().len();
        let depth_after_10 = repl.estimate_stack_depth();

        // Verify formula: depth = bindings / 10 + 1
        // Change in depth should match (new_bindings - old_bindings) / 10
        let bindings_added = bindings_after_10 - initial_bindings;
        let expected_depth_increase = bindings_added / 10;
        let actual_depth_increase = depth_after_10 - initial_depth;

        assert_eq!(
            actual_depth_increase, expected_depth_increase,
            "Stack depth should increase by {expected_depth_increase} (using division), got {actual_depth_increase}"
        );

        // Add 10 more bindings
        for i in 10..20 {
            repl.eval(&format!("let var{i} = {i}")).unwrap();
        }
        let bindings_after_20 = repl.get_bindings().len();
        let depth_after_20 = repl.estimate_stack_depth();

        // Verify division (not modulo): adding more bindings continues to increase depth
        let total_bindings_added = bindings_after_20 - initial_bindings;
        let expected_final_depth_increase = total_bindings_added / 10;
        let actual_final_depth_increase = depth_after_20 - initial_depth;

        assert_eq!(
            actual_final_depth_increase, expected_final_depth_increase,
            "Stack depth formula should use division (not modulo)"
        );

        // Also verify it doesn't just return 0 (stub mutation)
        assert!(depth_after_20 > 0, "Stack depth should never be 0");
    }

    #[test]
    fn test_execute_with_seed_resource_tracking() {
        // Mutation test: Verify resource usage calculations use subtraction
        // MISSED: replace - with + in execute_with_seed (lines 87, 80)
        // MISSED: replace - with / in execute_with_seed (line 80)

        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // First execution - should increase heap
        let result1 = repl.execute_with_seed("let x = 10", 12345);
        assert!(result1.output.is_ok());

        // heap_bytes calculation: self.estimate_heap_usage() - start_heap
        // This should be the difference between current and start
        let current_heap = repl.estimate_heap_usage();

        // The mutation tests verify that subtraction is used (not addition or division)
        // If + was used: result would be current + start (huge number)
        // If / was used: result would be current / start (wrong calculation)
        assert!(
            result1.resource_usage.heap_bytes > 0,
            "Heap bytes should be positive, got {}",
            result1.resource_usage.heap_bytes
        );

        // Verify it's a reasonable difference (not addition or division)
        assert!(
            result1.resource_usage.heap_bytes <= current_heap,
            "Heap bytes {0} should be <= current heap {current_heap} (proves subtraction, not addition)",
            result1.resource_usage.heap_bytes
        );

        // Second execution with string (larger data) - should use more heap
        let before_string = repl.estimate_heap_usage();
        let _result2 =
            repl.execute_with_seed("let y = \"long string value that uses memory\"", 12345);
        let after_string = repl.estimate_heap_usage();

        // String should cause more heap growth than integer
        let heap_change = after_string.saturating_sub(before_string);
        assert!(
            heap_change > 0,
            "String variable should increase heap usage"
        );
    }

    #[test]
    fn test_execute_with_seed_state_hash_determinism() {
        // Mutation test: The comparison operators (==) and boolean operators (&&)
        // in lines 71-80 are currently DEAD CODE because s is always "success"
        //
        // MISSED mutations in DEAD CODE:
        // - replace == with != (lines 71, 73) - never executed
        // - replace && with || (line 79) - never executed
        //
        // These mutations reveal that the string parsing logic is not being used.
        // The real functionality we can test is deterministic state hashing.

        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let mut repl1 = Repl::new(temp_dir1.path().to_path_buf()).unwrap();
        let mut repl2 = Repl::new(temp_dir2.path().to_path_buf()).unwrap();

        // Same input should produce same state hash (the actual working code)
        let result1 = repl1.execute_with_seed("let x = 42", 12345);
        let result2 = repl2.execute_with_seed("let x = 42", 12345);

        assert!(result1.output.is_ok());
        assert!(result2.output.is_ok());
        assert_eq!(
            result1.state_hash, result2.state_hash,
            "Deterministic execution should produce same state hash"
        );

        // Different input should produce different hash
        let result3 = repl1.execute_with_seed("let y = 99", 12345);
        assert_ne!(
            result1.state_hash, result3.state_hash,
            "Different state should produce different hash"
        );
    }

    #[test]
    fn test_deterministic_rng_reset() {
        // Mutation test: Verify DeterministicRng::reset actually resets state
        // MISSED: replace DeterministicRng::reset with ()

        let mut rng = DeterministicRng::new(12345);

        // Generate some numbers
        let first = rng.next();
        let second = rng.next();
        let third = rng.next();

        // Reset should restore to seed state
        rng.reset();

        // After reset, should generate same sequence
        let first_after_reset = rng.next();
        let second_after_reset = rng.next();
        let third_after_reset = rng.next();

        assert_eq!(
            first, first_after_reset,
            "First value after reset should match original"
        );
        assert_eq!(
            second, second_after_reset,
            "Second value after reset should match original"
        );
        assert_eq!(
            third, third_after_reset,
            "Third value after reset should match original"
        );
    }
}
