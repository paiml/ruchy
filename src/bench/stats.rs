//! Statistical analysis utilities for benchmarking
//!
//! Provides standard statistical functions for analyzing benchmark results:
//! - Mean, median, mode
//! - Standard deviation, variance
//! - Percentiles
//! - Min, max, range
//!
//! # Examples
//!
//! ```
//! use ruchy::bench::stats::Statistics;
//! use std::time::Duration;
//!
//! let times = vec![
//!     Duration::from_millis(10),
//!     Duration::from_millis(20),
//!     Duration::from_millis(30),
//! ];
//!
//! let stats = Statistics::from_durations(&times);
//! println!("Mean: {:.2}ms", stats.mean().as_secs_f64() * 1000.0);
//! println!("StdDev: {:.2}ms", stats.std_dev().as_secs_f64() * 1000.0);
//! ```

use std::time::Duration;

/// Statistical analysis of benchmark durations
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Sorted duration samples
    samples: Vec<Duration>,
}

impl Statistics {
    /// Create statistics from duration samples
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::stats::Statistics;
    /// use std::time::Duration;
    ///
    /// let times = vec![Duration::from_millis(10), Duration::from_millis(20)];
    /// let stats = Statistics::from_durations(&times);
    /// ```
    #[must_use]
    pub fn from_durations(durations: &[Duration]) -> Self {
        let mut samples = durations.to_vec();
        samples.sort();
        Self { samples }
    }

    /// Get mean (average) duration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::stats::Statistics;
    /// use std::time::Duration;
    ///
    /// let times = vec![Duration::from_millis(10), Duration::from_millis(30)];
    /// let stats = Statistics::from_durations(&times);
    /// assert_eq!(stats.mean(), Duration::from_millis(20));
    /// ```
    #[must_use]
    pub fn mean(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }

        let sum: Duration = self.samples.iter().sum();
        sum / self.samples.len() as u32
    }

    /// Get median duration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::stats::Statistics;
    /// use std::time::Duration;
    ///
    /// let times = vec![
    ///     Duration::from_millis(10),
    ///     Duration::from_millis(20),
    ///     Duration::from_millis(30),
    /// ];
    /// let stats = Statistics::from_durations(&times);
    /// assert_eq!(stats.median(), Duration::from_millis(20));
    /// ```
    #[must_use]
    pub fn median(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }

        let mid = self.samples.len() / 2;
        if self.samples.len().is_multiple_of(2) {
            // Even number of samples - average the two middle values
            let sum = self.samples[mid - 1] + self.samples[mid];
            sum / 2
        } else {
            // Odd number of samples - take middle value
            self.samples[mid]
        }
    }

    /// Get minimum duration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::stats::Statistics;
    /// use std::time::Duration;
    ///
    /// let times = vec![Duration::from_millis(10), Duration::from_millis(30)];
    /// let stats = Statistics::from_durations(&times);
    /// assert_eq!(stats.min(), Duration::from_millis(10));
    /// ```
    #[must_use]
    pub fn min(&self) -> Duration {
        self.samples.first().copied().unwrap_or(Duration::ZERO)
    }

    /// Get maximum duration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::stats::Statistics;
    /// use std::time::Duration;
    ///
    /// let times = vec![Duration::from_millis(10), Duration::from_millis(30)];
    /// let stats = Statistics::from_durations(&times);
    /// assert_eq!(stats.max(), Duration::from_millis(30));
    /// ```
    #[must_use]
    pub fn max(&self) -> Duration {
        self.samples.last().copied().unwrap_or(Duration::ZERO)
    }

    /// Get standard deviation
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::stats::Statistics;
    /// use std::time::Duration;
    ///
    /// let times = vec![Duration::from_millis(10), Duration::from_millis(30)];
    /// let stats = Statistics::from_durations(&times);
    /// // StdDev should be ~10ms
    /// assert!(stats.std_dev().as_millis() >= 9 && stats.std_dev().as_millis() <= 11);
    /// ```
    #[must_use]
    pub fn std_dev(&self) -> Duration {
        if self.samples.len() < 2 {
            return Duration::ZERO;
        }

        let mean = self.mean().as_secs_f64();
        let variance: f64 = self
            .samples
            .iter()
            .map(|d| {
                let diff = d.as_secs_f64() - mean;
                diff * diff
            })
            .sum::<f64>()
            / (self.samples.len() - 1) as f64;

        Duration::from_secs_f64(variance.sqrt())
    }

    /// Get percentile value
    ///
    /// # Arguments
    ///
    /// * `p` - Percentile (0.0-100.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::stats::Statistics;
    /// use std::time::Duration;
    ///
    /// let times = vec![
    ///     Duration::from_millis(10),
    ///     Duration::from_millis(20),
    ///     Duration::from_millis(30),
    ///     Duration::from_millis(40),
    ///     Duration::from_millis(50),
    /// ];
    /// let stats = Statistics::from_durations(&times);
    /// assert_eq!(stats.percentile(50.0), Duration::from_millis(30));
    /// ```
    #[must_use]
    pub fn percentile(&self, p: f64) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }

        let index = ((p / 100.0) * self.samples.len() as f64) as usize;
        self.samples[index.min(self.samples.len() - 1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test mean calculation
    #[test]
    fn test_mean() {
        let times = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
        ];
        let stats = Statistics::from_durations(&times);
        assert_eq!(stats.mean(), Duration::from_millis(20));
    }

    /// Test median calculation with odd number of samples
    #[test]
    fn test_median_odd() {
        let times = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
        ];
        let stats = Statistics::from_durations(&times);
        assert_eq!(stats.median(), Duration::from_millis(20));
    }

    /// Test median calculation with even number of samples
    #[test]
    fn test_median_even() {
        let times = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(40),
        ];
        let stats = Statistics::from_durations(&times);
        assert_eq!(stats.median(), Duration::from_millis(25));
    }

    /// Test min/max
    #[test]
    fn test_min_max() {
        let times = vec![
            Duration::from_millis(30),
            Duration::from_millis(10),
            Duration::from_millis(20),
        ];
        let stats = Statistics::from_durations(&times);
        assert_eq!(stats.min(), Duration::from_millis(10));
        assert_eq!(stats.max(), Duration::from_millis(30));
    }

    /// Test standard deviation
    #[test]
    fn test_std_dev() {
        let times = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
        ];
        let stats = Statistics::from_durations(&times);

        // Expected stddev for [10, 20, 30] is 10ms
        let stddev_ms = stats.std_dev().as_millis();
        assert!((9..=11).contains(&stddev_ms), "StdDev should be ~10ms, got {stddev_ms}");
    }

    /// Test percentiles
    #[test]
    fn test_percentiles() {
        let times = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(40),
            Duration::from_millis(50),
        ];
        let stats = Statistics::from_durations(&times);

        assert_eq!(stats.percentile(0.0), Duration::from_millis(10));
        assert_eq!(stats.percentile(50.0), Duration::from_millis(30));
        assert_eq!(stats.percentile(100.0), Duration::from_millis(50));
    }

    /// Test empty samples edge case
    #[test]
    fn test_empty_samples() {
        let times: Vec<Duration> = vec![];
        let stats = Statistics::from_durations(&times);

        assert_eq!(stats.mean(), Duration::ZERO);
        assert_eq!(stats.median(), Duration::ZERO);
        assert_eq!(stats.min(), Duration::ZERO);
        assert_eq!(stats.max(), Duration::ZERO);
        assert_eq!(stats.std_dev(), Duration::ZERO);
        assert_eq!(stats.percentile(50.0), Duration::ZERO);
    }

    /// Property test: Mean should be between min and max
    #[test]
    #[ignore = "Property test - run with: cargo test -- --ignored"]
    fn prop_mean_bounds() {
        use proptest::prelude::*;

        proptest!(|(times in prop::collection::vec(0u64..10000, 2..100))| {
            let durations: Vec<Duration> = times.iter()
                .map(|&t| Duration::from_millis(t))
                .collect();

            let stats = Statistics::from_durations(&durations);

            prop_assert!(stats.mean() >= stats.min());
            prop_assert!(stats.mean() <= stats.max());
        });
    }

    /// Property test: Median should be between min and max
    #[test]
    #[ignore = "Property test - run with: cargo test -- --ignored"]
    fn prop_median_bounds() {
        use proptest::prelude::*;

        proptest!(|(times in prop::collection::vec(0u64..10000, 2..100))| {
            let durations: Vec<Duration> = times.iter()
                .map(|&t| Duration::from_millis(t))
                .collect();

            let stats = Statistics::from_durations(&durations);

            prop_assert!(stats.median() >= stats.min());
            prop_assert!(stats.median() <= stats.max());
        });
    }
}
