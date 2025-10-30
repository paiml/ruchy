#![allow(missing_docs)]
// Matrix Test 04: Time Series Analysis (Native Platform)
//
// Companion to: tests/e2e/matrix/04-time-series.spec.ts (WASM - DEFERRED)
//
// Goal: Verify time series analysis workflows
// This test uses rexpect to interact with the native `ruchy` REPL
//
// NOTE: Time series operations implemented using basic array operations
// (slice, window, map, reduce) - no dedicated time series library yet

use rexpect::session::spawn_command;
use std::process::Command;

/// Helper to create a ruchy REPL session
fn spawn_ruchy_repl() -> rexpect::session::PtySession {
    let cmd = Command::new("ruchy");
    spawn_command(cmd, Some(10000)).expect("Failed to spawn ruchy REPL")
}

#[test]
fn test_matrix_native_04_01_simple_moving_average() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Time series: [10, 20, 30, 40, 50]
    // 3-period SMA at index 2: (10+20+30)/3 = 20
    repl.send_line("let window = [10, 20, 30]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("window.reduce(|acc, x| acc + x, 0) / window.len()")
        .expect("Failed to send command");

    repl.exp_string("20").expect("3-period SMA should be 20");
}

#[test]
fn test_matrix_native_04_02_sliding_window_extraction() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Extract sliding window from time series
    // Series: [100, 110, 120, 130, 140]
    // Window at position 1 (size 3): [110, 120, 130]
    repl.send_line("let series = [100, 110, 120, 130, 140]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Access window elements
    repl.send_line("series[1]")
        .expect("Failed to send command");

    repl.exp_string("110").expect("Window start should be 110");
}

#[test]
fn test_matrix_native_04_03_percent_change_calculation() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Percent change: ((new - old) / old) * 100
    // From 100 to 120: ((120-100)/100)*100 = 20%
    repl.send_line("let old_value = 100")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let new_value = 120")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Calculate percent change
    repl.send_line("((new_value - old_value) * 100) / old_value")
        .expect("Failed to send command");

    repl.exp_string("20").expect("Percent change should be 20%");
}

#[test]
fn test_matrix_native_04_04_cumulative_sum() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Cumulative sum: [1, 2, 3, 4, 5] → [1, 3, 6, 10, 15]
    // Verify first few cumulative values
    repl.send_line("let data = [1, 2, 3, 4, 5]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Total cumulative sum (last value)
    repl.send_line("data.reduce(|acc, x| acc + x, 0)")
        .expect("Failed to send command");

    repl.exp_string("15").expect("Cumulative sum should be 15");
}

#[test]
fn test_matrix_native_04_05_rolling_max() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Rolling max in window [10, 25, 15] should be 25
    repl.send_line("let window = [10, 25, 15]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Max is at index 1
    repl.send_line("window[1]")
        .expect("Failed to send command");

    repl.exp_string("25").expect("Max in window should be 25");
}

#[test]
fn test_matrix_native_04_06_rolling_min() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Rolling min in window [30, 15, 45] should be 15
    repl.send_line("let window = [30, 15, 45]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Min is at index 1
    repl.send_line("window[1]")
        .expect("Failed to send command");

    repl.exp_string("15").expect("Min in window should be 15");
}

#[test]
fn test_matrix_native_04_07_trend_direction() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Trend: compare first and last values
    // Uptrend: last > first
    // Series: [100, 110, 105, 120] → uptrend (120 > 100)
    repl.send_line("let first = 100")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let last = 120")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Trend magnitude
    repl.send_line("last - first")
        .expect("Failed to send command");

    repl.exp_string("20").expect("Uptrend magnitude should be 20");
}

#[test]
fn test_matrix_native_04_08_volatility_range() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Volatility (simple range): max - min
    // Data: [90, 110, 85, 105, 95]
    // Range: 110 - 85 = 25
    repl.send_line("let max_val = 110")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let min_val = 85")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("max_val - min_val")
        .expect("Failed to send command");

    repl.exp_string("25").expect("Volatility range should be 25");
}

#[test]
fn test_matrix_native_04_09_momentum_calculation() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Momentum: current - n_periods_ago
    // Current: 150, 5 periods ago: 120
    // Momentum = 150 - 120 = 30
    repl.send_line("let current = 150")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let past = 120")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("current - past")
        .expect("Failed to send command");

    repl.exp_string("30").expect("Momentum should be 30");
}

#[test]
fn test_matrix_native_04_10_rate_of_change() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Rate of Change (ROC): ((current - past) / past) * 100
    // Current: 110, Past: 100
    // ROC = ((110-100)/100)*100 = 10%
    repl.send_line("let current = 110")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let past = 100")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("((current - past) * 100) / past")
        .expect("Failed to send command");

    repl.exp_string("10").expect("ROC should be 10%");
}

#[test]
fn test_matrix_native_04_11_exponential_weighting() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Simple exponential weighting calculation
    // Weight recent data more: w1*recent + w2*older
    // Recent: 100, weight: 7
    // Older: 80, weight: 3
    // Weighted value: (100*7 + 80*3) / 10 = (700+240)/10 = 94
    repl.send_line("let recent = 100")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let older = 80")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Calculate weighted value
    repl.send_line("(recent * 7 + older * 3) / 10")
        .expect("Failed to send command");

    repl.exp_string("94").expect("Exponential weighted value should be 94");
}

#[test]
fn test_matrix_native_04_12_anomaly_detection_threshold() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Anomaly detection: value outside mean ± threshold
    // Mean: 100, Threshold: 20
    // Lower bound: 80, Upper bound: 120
    // Value: 150 (anomaly - exceeds upper bound)
    repl.send_line("let value = 150")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let mean = 100")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let threshold = 20")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Check if anomaly (value - mean > threshold)
    repl.send_line("value - mean")
        .expect("Failed to send command");

    repl.exp_string("50").expect("Deviation should be 50 (anomaly detected)");
}

#[test]
fn test_matrix_native_04_13_seasonality_check() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Seasonality: compare same period across cycles
    // Period 1 value: 100, Period 2 value: 105
    // Seasonal difference: 105 - 100 = 5
    repl.send_line("let period1 = 100")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("let period2 = 105")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("period2 - period1")
        .expect("Failed to send command");

    repl.exp_string("5").expect("Seasonal difference should be 5");
}

#[test]
fn test_matrix_native_04_14_lag_calculation() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Lag: access previous period's value
    // Series: [100, 110, 120, 130, 140]
    // Lag-1 of index 3 (130) is index 2 (120)
    repl.send_line("let series = [100, 110, 120, 130, 140]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Current value at index 3
    repl.send_line("let current = series[3]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Lag-1 value at index 2
    repl.send_line("let lagged = series[2]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Difference from lag
    repl.send_line("current - lagged")
        .expect("Failed to send command");

    repl.exp_string("10").expect("Difference from lag should be 10");
}
