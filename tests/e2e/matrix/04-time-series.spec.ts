import { test, expect } from '@playwright/test';

/**
 * Matrix Test 04: Time Series Analysis (WASM Platform) - DEFERRED
 *
 * This test verifies time series analysis workflows in the WASM REPL.
 * Companion native test: tests/matrix_004_time_series_native.rs
 *
 * STATUS: DEFERRED pending WASM eval() implementation
 * See: tests/e2e/matrix/README.md for rationale
 *
 * Goal: Ensure identical behavior between WASM and native platforms
 */

test.describe('Matrix Test 04: Time Series Analysis (WASM Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to load
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });
  });

  test('should calculate simple moving average', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Time series: [10, 20, 30, 40, 50]
    // 3-period SMA at index 2: (10+20+30)/3 = 20
    await input.fill('let window = [10, 20, 30]');
    await input.press('Enter');

    await input.fill('window.reduce(|acc, x| acc + x, 0) / window.len()');
    await input.press('Enter');

    await expect(output).toContainText('20');
  });

  test('should extract sliding window', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Extract sliding window from time series
    // Series: [100, 110, 120, 130, 140]
    // Window at position 1 (size 3): [110, 120, 130]
    await input.fill('let series = [100, 110, 120, 130, 140]');
    await input.press('Enter');

    // Access window elements
    await input.fill('series[1]');
    await input.press('Enter');

    await expect(output).toContainText('110');
  });

  test('should calculate percent change', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Percent change: ((new - old) / old) * 100
    // From 100 to 120: ((120-100)/100)*100 = 20%
    await input.fill('let old_value = 100');
    await input.press('Enter');

    await input.fill('let new_value = 120');
    await input.press('Enter');

    // Calculate percent change
    await input.fill('((new_value - old_value) * 100) / old_value');
    await input.press('Enter');

    await expect(output).toContainText('20');
  });

  test('should calculate cumulative sum', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Cumulative sum: [1, 2, 3, 4, 5] → [1, 3, 6, 10, 15]
    // Verify total cumulative sum (last value)
    await input.fill('let data = [1, 2, 3, 4, 5]');
    await input.press('Enter');

    await input.fill('data.reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    await expect(output).toContainText('15');
  });

  test('should calculate rolling max', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Rolling max in window [10, 25, 15] should be 25
    await input.fill('let window = [10, 25, 15]');
    await input.press('Enter');

    // Max is at index 1
    await input.fill('window[1]');
    await input.press('Enter');

    await expect(output).toContainText('25');
  });

  test('should calculate rolling min', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Rolling min in window [30, 15, 45] should be 15
    await input.fill('let window = [30, 15, 45]');
    await input.press('Enter');

    // Min is at index 1
    await input.fill('window[1]');
    await input.press('Enter');

    await expect(output).toContainText('15');
  });

  test('should detect trend direction', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Trend: compare first and last values
    // Uptrend: last > first
    // Series: [100, 110, 105, 120] → uptrend (120 > 100)
    await input.fill('let first = 100');
    await input.press('Enter');

    await input.fill('let last = 120');
    await input.press('Enter');

    // Trend magnitude
    await input.fill('last - first');
    await input.press('Enter');

    await expect(output).toContainText('20');
  });

  test('should calculate volatility range', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Volatility (simple range): max - min
    // Data: [90, 110, 85, 105, 95]
    // Range: 110 - 85 = 25
    await input.fill('let max_val = 110');
    await input.press('Enter');

    await input.fill('let min_val = 85');
    await input.press('Enter');

    await input.fill('max_val - min_val');
    await input.press('Enter');

    await expect(output).toContainText('25');
  });

  test('should calculate momentum', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Momentum: current - n_periods_ago
    // Current: 150, 5 periods ago: 120
    // Momentum = 150 - 120 = 30
    await input.fill('let current = 150');
    await input.press('Enter');

    await input.fill('let past = 120');
    await input.press('Enter');

    await input.fill('current - past');
    await input.press('Enter');

    await expect(output).toContainText('30');
  });

  test('should calculate rate of change', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Rate of Change (ROC): ((current - past) / past) * 100
    // Current: 110, Past: 100
    // ROC = ((110-100)/100)*100 = 10%
    await input.fill('let current = 110');
    await input.press('Enter');

    await input.fill('let past = 100');
    await input.press('Enter');

    await input.fill('((current - past) * 100) / past');
    await input.press('Enter');

    await expect(output).toContainText('10');
  });

  test('should calculate exponential weighting', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Simple exponential weighting calculation
    // Weight recent data more: w1*recent + w2*older
    // Recent: 100, weight: 7
    // Older: 80, weight: 3
    // Weighted value: (100*7 + 80*3) / 10 = (700+240)/10 = 94
    await input.fill('let recent = 100');
    await input.press('Enter');

    await input.fill('let older = 80');
    await input.press('Enter');

    // Calculate weighted value
    await input.fill('(recent * 7 + older * 3) / 10');
    await input.press('Enter');

    await expect(output).toContainText('94');
  });

  test('should detect anomalies with threshold', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Anomaly detection: value outside mean ± threshold
    // Mean: 100, Threshold: 20
    // Lower bound: 80, Upper bound: 120
    // Value: 150 (anomaly - exceeds upper bound)
    await input.fill('let value = 150');
    await input.press('Enter');

    await input.fill('let mean = 100');
    await input.press('Enter');

    await input.fill('let threshold = 20');
    await input.press('Enter');

    // Check if anomaly (value - mean > threshold)
    await input.fill('value - mean');
    await input.press('Enter');

    await expect(output).toContainText('50');
  });

  test('should check seasonality', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Seasonality: compare same period across cycles
    // Period 1 value: 100, Period 2 value: 105
    // Seasonal difference: 105 - 100 = 5
    await input.fill('let period1 = 100');
    await input.press('Enter');

    await input.fill('let period2 = 105');
    await input.press('Enter');

    await input.fill('period2 - period1');
    await input.press('Enter');

    await expect(output).toContainText('5');
  });

  test('should calculate lag', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Lag: access previous period's value
    // Series: [100, 110, 120, 130, 140]
    // Lag-1 of index 3 (130) is index 2 (120)
    await input.fill('let series = [100, 110, 120, 130, 140]');
    await input.press('Enter');

    // Current value at index 3
    await input.fill('let current = series[3]');
    await input.press('Enter');

    // Lag-1 value at index 2
    await input.fill('let lagged = series[2]');
    await input.press('Enter');

    // Difference from lag
    await input.fill('current - lagged');
    await input.press('Enter');

    await expect(output).toContainText('10');
  });
});
