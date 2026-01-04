//! Reproducibility infrastructure for deterministic execution.
//!
//! This module provides centralized random seed management to ensure
//! reproducible results across all Ruchy components.
//!
//! # Usage
//!
//! ```rust
//! use ruchy::reproducibility::{get_seed, get_rng};
//!
//! // Get the global seed
//! let seed = get_seed();
//!
//! // Get a seeded RNG for a specific component
//! let mut rng = get_rng("parser");
//! ```

use std::sync::LazyLock;

/// Default seed for reproducibility (the answer to everything)
pub const DEFAULT_SEED: u64 = 42;

/// Global seed, configurable via `RUCHY_SEED` environment variable
pub static GLOBAL_SEED: LazyLock<u64> = LazyLock::new(|| {
    std::env::var("RUCHY_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SEED)
});

/// Get the global random seed.
///
/// This reads from the `RUCHY_SEED` environment variable if set,
/// otherwise returns the default seed (42).
#[inline]
pub fn get_seed() -> u64 {
    *GLOBAL_SEED
}

/// Get a component-specific seed derived from the global seed.
///
/// This ensures different components get different but deterministic
/// seed values while maintaining overall reproducibility.
pub fn get_component_seed(component: &str) -> u64 {
    let base = get_seed();
    let hash = component.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    });
    base.wrapping_add(hash)
}

/// Get a seeded random number generator for a component.
///
/// Uses the global seed combined with component name for reproducibility.
pub fn get_rng(component: &str) -> SimpleRng {
    SimpleRng::new(get_component_seed(component))
}

/// Simple deterministic RNG for when rand crate is not available.
///
/// Uses a linear congruential generator (LCG) with the same parameters
/// as glibc for compatibility.
#[derive(Debug, Clone)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    /// Create a new RNG with the given seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Create a new RNG using the global seed.
    pub fn from_global_seed() -> Self {
        Self::new(get_seed())
    }

    /// Create a new RNG for a specific component.
    pub fn for_component(component: &str) -> Self {
        Self::new(get_component_seed(component))
    }

    /// Generate the next random u64.
    pub fn next_u64(&mut self) -> u64 {
        // LCG parameters from Knuth MMIX
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Generate a random number in [0, max).
    pub fn next_bounded(&mut self, max: u64) -> u64 {
        self.next_u64() % max
    }

    /// Reset the RNG to a new seed.
    pub fn reseed(&mut self, seed: u64) {
        self.state = seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_seed() {
        // Same seed should produce same sequence
        let mut rng1 = SimpleRng::new(42);
        let mut rng2 = SimpleRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_component_seeds_differ() {
        let seed1 = get_component_seed("parser");
        let seed2 = get_component_seed("oracle");
        assert_ne!(
            seed1, seed2,
            "Different components should have different seeds"
        );
    }

    #[test]
    fn test_component_seeds_deterministic() {
        let seed1 = get_component_seed("parser");
        let seed2 = get_component_seed("parser");
        assert_eq!(seed1, seed2, "Same component should always get same seed");
    }

    #[test]
    fn test_rng_bounded() {
        let mut rng = SimpleRng::new(42);
        for _ in 0..1000 {
            let val = rng.next_bounded(100);
            assert!(val < 100);
        }
    }
}
