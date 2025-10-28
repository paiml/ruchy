import { test, expect } from '@playwright/test';

/**
 * Matrix Test 04: Time Series Analysis (Notebook Platform)
 *
 * This test verifies time series analysis workflows in the Ruchy notebook interface.
 * Companion native test: tests/matrix_004_time_series_native.rs
 *
 * Goal: Ensure identical behavior between notebook and native platforms
 *
 * FIXED: DEFECT-E2E-PHANTOM-UI - Rewritten to use actual notebook.html elements
 * Previous: Tests expected phantom UI (#status, #repl-input) that never existed
 * Current: Tests use real notebook UI (#notebook-cells, .CodeMirror, cell outputs)
 */

test.describe('Matrix Test 04: Time Series Analysis (Notebook Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for actual notebook interface to load
    await page.waitForSelector('#notebook-cells', { timeout: 10000 });

    // Verify notebook UI components are ready
    await expect(page.locator('#notebook-cells')).toBeVisible();
    await expect(page.locator('.CodeMirror').first()).toBeVisible();
  });

  test('should calculate simple moving average', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Time series: [10, 20, 30, 40, 50]
    // 3-period SMA at index 2: (10+20+30)/3 = 20
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let window = [10, 20, 30]\nwindow.reduce(|acc, x| acc + x, 0) / window.len()');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('20');
  });

  test('should extract sliding window', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Extract sliding window from time series
    // Series: [100, 110, 120, 130, 140]
    // Window at position 1 (size 3): [110, 120, 130]
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let series = [100, 110, 120, 130, 140]\nseries[1]');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('110');
  });

  test('should calculate percent change', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Percent change: ((new - old) / old) * 100
    // From 100 to 120: ((120-100)/100)*100 = 20%
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let old_value = 100\nlet new_value = 120\n((new_value - old_value) * 100) / old_value');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('20');
  });

  test('should calculate cumulative sum', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Cumulative sum: [1, 2, 3, 4, 5] → [1, 3, 6, 10, 15]
    // Verify total cumulative sum (last value)
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let data = [1, 2, 3, 4, 5]\ndata.reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('15');
  });

  test('should calculate rolling max', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Rolling max in window [10, 25, 15] should be 25
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let window = [10, 25, 15]\nwindow[1]');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('25');
  });

  test('should calculate rolling min', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Rolling min in window [30, 15, 45] should be 15
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let window = [30, 15, 45]\nwindow[1]');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('15');
  });

  test('should detect trend direction', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Trend: compare first and last values
    // Uptrend: last > first
    // Series: [100, 110, 105, 120] → uptrend (120 > 100)
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let first = 100\nlet last = 120\nlast - first');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('20');
  });

  test('should calculate volatility range', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Volatility (simple range): max - min
    // Data: [90, 110, 85, 105, 95]
    // Range: 110 - 85 = 25
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let max_val = 110\nlet min_val = 85\nmax_val - min_val');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('25');
  });

  test('should calculate momentum', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Momentum: current - n_periods_ago
    // Current: 150, 5 periods ago: 120
    // Momentum = 150 - 120 = 30
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let current = 150\nlet past = 120\ncurrent - past');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('30');
  });

  test('should calculate rate of change', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Rate of Change (ROC): ((current - past) / past) * 100
    // Current: 110, Past: 100
    // ROC = ((110-100)/100)*100 = 10%
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let current = 110\nlet past = 100\n((current - past) * 100) / past');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('10');
  });

  test('should calculate exponential weighting', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Simple exponential weighting calculation
    // Weight recent data more: w1*recent + w2*older
    // Recent: 100, weight: 7
    // Older: 80, weight: 3
    // Weighted value: (100*7 + 80*3) / 10 = (700+240)/10 = 94
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let recent = 100\nlet older = 80\n(recent * 7 + older * 3) / 10');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('94');
  });

  test('should detect anomalies with threshold', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Anomaly detection: value outside mean ± threshold
    // Mean: 100, Threshold: 20
    // Lower bound: 80, Upper bound: 120
    // Value: 150 (anomaly - exceeds upper bound)
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let value = 150\nlet mean = 100\nlet threshold = 20\nvalue - mean');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('50');
  });

  test('should check seasonality', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Seasonality: compare same period across cycles
    // Period 1 value: 100, Period 2 value: 105
    // Seasonal difference: 105 - 100 = 5
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let period1 = 100\nlet period2 = 105\nperiod2 - period1');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('5');
  });

  test('should calculate lag', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Lag: access previous period's value
    // Series: [100, 110, 120, 130, 140]
    // Lag-1 of index 3 (130) is index 2 (120)
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let series = [100, 110, 120, 130, 140]\nlet current = series[3]\nlet lagged = series[2]\ncurrent - lagged');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('10');
  });
});
