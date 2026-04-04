#![allow(missing_docs)]
//! EMBED-006: Release-mode startup latency benchmark.
//!
//! Validates parent spec Section 10 criterion #2: `Engine::new()` must
//! complete in < 5ms on release builds (x86_64). The test is `#[ignore]`'d
//! by default because debug-mode startup exceeds the budget; CI invokes it
//! with `cargo test --release --test startup_benchmark -- --ignored`.
//!
//! Ticket: EMBED-006 (Pillar 9 sub-spec Section 7).

use ruchy_embed::Engine;
use std::time::Duration;

/// Parent spec threshold for `Engine::new()` on release builds.
const RELEASE_STARTUP_BUDGET: Duration = Duration::from_millis(5);

/// Sample count for steady-state measurement (first run may warm caches).
const SAMPLES: usize = 8;

#[test]
#[ignore = "release-only benchmark; run with `cargo test --release -- --ignored`"]
fn test_embed_006_engine_new_under_5ms_release() {
    // Warm up (first engine construction may pay one-time costs).
    let _warm = Engine::new();

    // Measure steady-state startup. The MINIMUM across samples is the right
    // metric for "how fast can this be"; the MAX catches pathological cases.
    let mut min = Duration::MAX;
    let mut max = Duration::ZERO;
    for _ in 0..SAMPLES {
        let engine = Engine::new();
        let t = engine.startup_time();
        if t < min {
            min = t;
        }
        if t > max {
            max = t;
        }
    }

    assert!(
        max < RELEASE_STARTUP_BUDGET,
        "Engine::new() exceeded 5ms budget: min={min:?}, max={max:?}"
    );
}

#[test]
fn test_embed_006_startup_time_is_reported() {
    // This test is NOT release-gated: it only asserts the API returns a
    // non-default value (Engine records startup_time via Instant::elapsed).
    let engine = Engine::new();
    let t = engine.startup_time();
    assert!(
        t > Duration::ZERO,
        "startup_time() must be > 0, got {t:?}"
    );
    // And reasonable even in debug mode (generous upper bound).
    assert!(
        t < Duration::from_secs(1),
        "startup_time() must be < 1s even in debug, got {t:?}"
    );
}
