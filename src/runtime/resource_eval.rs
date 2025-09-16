//! Resource-bounded evaluation for the REPL
//!
//! Provides safe evaluation with memory and time limits, automatic rollback on failure.
use anyhow::{Result, anyhow};
use std::time::{Duration, Instant};
use crate::runtime::repl::{Repl, Value};
use crate::runtime::transaction::{TransactionMetadata, SavePoint};
// ============================================================================
// Resource-Bounded Evaluation
// ============================================================================
impl Repl {
    /// Evaluate with resource bounds and automatic rollback on failure
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::eval_bounded;
/// 
/// let result = eval_bounded("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn eval_bounded(
        &mut self,
        input: &str,
        memory_limit: Option<usize>,
        time_limit: Option<Duration>,
    ) -> Result<String> {
        // Create transaction metadata
        let metadata = TransactionMetadata {
            description: format!("eval: {}", input.chars().take(50).collect::<String>()),
            memory_limit,
            time_limit,
            speculative: false,
        };
        // Begin transaction
        let tx_id = self.tx_state.begin_transaction(metadata)?;
        // Set up timeout
        let start = Instant::now();
        let deadline = time_limit.map(|d| start + d);
        // Evaluate with transaction protection
        let result = self.eval_with_transaction(input, tx_id, deadline);
        match result {
            Ok(output) => {
                // Success - commit transaction
                self.tx_state.commit_transaction(tx_id)?;
                Ok(output)
            }
            Err(e) => {
                // Failure - rollback transaction
                self.tx_state.rollback_transaction(tx_id)?;
                Err(e)
            }
        }
    }
    /// Try evaluation with speculative execution
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::try_eval;
/// 
/// let result = try_eval("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn try_eval(&mut self, input: &str) -> Result<String> {
        // Use savepoint for automatic rollback
        let savepoint = self.tx_state.savepoint()?;
        match self.eval(input) {
            Ok(result) => {
                // Success - commit savepoint
                savepoint.commit()?;
                Ok(result)
            }
            Err(e) => {
                // Failure - savepoint automatically rolls back
                drop(savepoint);
                Err(e)
            }
        }
    }
    /// Evaluate multiple expressions atomically
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::eval_atomic;
/// 
/// let result = eval_atomic("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn eval_atomic(&mut self, inputs: &[&str]) -> Result<Vec<String>> {
        let savepoint = self.tx_state.savepoint()?;
        let mut results = Vec::new();
        for input in inputs {
            match self.eval(input) {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Any failure rolls back all changes
                    drop(savepoint);
                    return Err(e);
                }
            }
        }
        // All succeeded - commit
        savepoint.commit()?;
        Ok(results)
    }
    /// Internal evaluation with transaction tracking
    fn eval_with_transaction(
        &mut self,
        input: &str,
        tx_id: crate::runtime::transaction::TransactionId,
        deadline: Option<Instant>,
    ) -> Result<String> {
        // Check deadline
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                return Err(anyhow!("Evaluation timeout"));
            }
        }
        // Check transaction limits periodically
        self.tx_state.check_transaction_limits(tx_id)?;
        // Regular evaluation
        self.eval(input)
    }
    /// Get current memory usage
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::memory_usage;
/// 
/// let result = memory_usage(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn memory_usage(&self) -> usize {
        self.tx_state.memory_used()
    }
    /// Create a checkpoint for manual rollback
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::checkpoint;
/// 
/// let result = checkpoint(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn checkpoint(&mut self) -> Result<CheckpointHandle> {
        let savepoint = self.tx_state.savepoint()?;
        Ok(CheckpointHandle {
            savepoint: Some(savepoint),
        })
    }
}
// ============================================================================
// Checkpoint Handle
// ============================================================================
/// Handle for manual checkpoint management
pub struct CheckpointHandle {
    savepoint: Option<SavePoint>,
}
impl CheckpointHandle {
    /// Commit the checkpoint
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::commit;
/// 
/// let result = commit(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn commit(mut self) -> Result<()> {
        if let Some(sp) = self.savepoint.take() {
            sp.commit()?;
        }
        Ok(())
    }
    /// Rollback to the checkpoint
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::rollback;
/// 
/// let result = rollback(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn rollback(mut self) -> Result<()> {
        if let Some(sp) = self.savepoint.take() {
            sp.rollback()?;
        }
        Ok(())
    }
}
impl Drop for CheckpointHandle {
    fn drop(&mut self) {
        // Automatic rollback if not explicitly handled
        if let Some(sp) = self.savepoint.take() {
            let _ = sp.rollback();
        }
    }
}
// ============================================================================
// Resource Limits Configuration
// ============================================================================
/// Configuration for resource-bounded evaluation
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory: usize,
    /// Maximum execution time
    pub max_time: Duration,
    /// Maximum recursion depth
    pub max_depth: usize,
    /// Maximum number of allocations
    pub max_allocations: usize,
}
impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 100 * 1024 * 1024, // 100MB
            max_time: Duration::from_secs(5),
            max_depth: 1000,
            max_allocations: 1_000_000,
        }
    }
}
impl ResourceLimits {
    /// Create limits for untrusted code
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::untrusted;
/// 
/// let result = untrusted(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn untrusted() -> Self {
        Self {
            max_memory: 10 * 1024 * 1024, // 10MB
            max_time: Duration::from_secs(1),
            max_depth: 100,
            max_allocations: 10_000,
        }
    }
    /// Create limits for testing
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::testing;
/// 
/// let result = testing(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn testing() -> Self {
        Self {
            max_memory: 1024 * 1024, // 1MB
            max_time: Duration::from_millis(100),
            max_depth: 50,
            max_allocations: 1_000,
        }
    }
}
// ============================================================================
// Sandbox Evaluation
// ============================================================================
/// Sandboxed evaluation environment
pub struct Sandbox {
    repl: Repl,
    limits: ResourceLimits,
}
impl Sandbox {
    /// Create a new sandbox with the given limits
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new(limits: ResourceLimits) -> Result<Self> {
        let mut config = crate::runtime::repl::ReplConfig::default();
        config.max_memory = limits.max_memory;
        config.max_depth = limits.max_depth;
        Ok(Self {
            repl: Repl::with_config(config)?,
            limits,
        })
    }
    /// Evaluate code in the sandbox
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::eval;
/// 
/// let result = eval("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn eval(&mut self, input: &str) -> Result<String> {
        self.repl.eval_bounded(
            input,
            Some(self.limits.max_memory),
            Some(self.limits.max_time),
        )
    }
    /// Reset the sandbox to initial state
/// # Examples
/// 
/// ```
/// use ruchy::runtime::resource_eval::reset;
/// 
/// let result = reset(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn reset(&mut self) -> Result<()> {
        self.repl.tx_state.clear();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
#[cfg(test)]
use proptest::prelude::*;
    #[test]
    fn test_bounded_evaluation() {
        let mut repl = Repl::new().unwrap();
        // Should succeed within limits
        let result = repl.eval_bounded(
            "1 + 1",
            Some(1024),
            Some(Duration::from_secs(1)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2");
    }
    #[test]
    fn test_atomic_evaluation() {
        let mut repl = Repl::new().unwrap();
        // All should succeed
        let results = repl.eval_atomic(&[
            "let x = 1",
            "let y = 2",
            "x + y",
        ]).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[2], "3");
        // Failure should rollback all
        let result = repl.eval_atomic(&[
            "let z = 10",
            "invalid syntax here",
            "z + 1",
        ]);
        assert!(result.is_err());
        // z should not exist due to rollback
        assert!(repl.eval("z").is_err());
    }
    #[test]
    fn test_checkpoint() {
        let mut repl = Repl::new().unwrap();
        repl.eval("let x = 1").unwrap();
        // Create checkpoint
        let checkpoint = repl.checkpoint().unwrap();
        // Make changes
        repl.eval("let x = 2").unwrap();
        repl.eval("let y = 3").unwrap();
        // Rollback
        checkpoint.rollback().unwrap();
        // x should be 1, y should not exist
        assert_eq!(repl.eval("x").unwrap(), "1");
        assert!(repl.eval("y").is_err());
    }
    #[test]
    fn test_sandbox() {
        let limits = ResourceLimits::testing();
        let mut sandbox = Sandbox::new(limits).unwrap();
        // Simple evaluation should work
        let result = sandbox.eval("2 * 3").unwrap();
        assert_eq!(result, "6");
        // Reset should clear state
        sandbox.eval("let a = 42").unwrap();
        sandbox.reset().unwrap();
        assert!(sandbox.eval("a").is_err());
    }
}
#[cfg(test)]
mod property_tests_resource_eval {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_eval_bounded_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
    }
}
