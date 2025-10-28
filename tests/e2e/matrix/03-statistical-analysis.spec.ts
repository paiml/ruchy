import { test, expect } from '@playwright/test';

/**
 * Matrix Test 03: Statistical Analysis (WASM Platform) - DEFERRED
 *
 * This test verifies statistical computation workflows in the WASM REPL.
 * Companion native test: tests/matrix_003_statistical_analysis_native.rs
 *
 * STATUS: DEFERRED pending WASM eval() implementation
 * See: tests/e2e/matrix/README.md for rationale
 *
 * Goal: Ensure identical behavior between WASM and native platforms
 */

test.describe('Matrix Test 03: Statistical Analysis (WASM Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to load
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });
  });

  test('should calculate mean', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Create dataset
    await input.fill('let data = [10, 20, 30, 40, 50]');
    await input.press('Enter');

    // Calculate mean: sum / count
    await input.fill('data.reduce(|acc, x| acc + x, 0) / data.len()');
    await input.press('Enter');

    // Mean = 150 / 5 = 30
    await expect(output).toContainText('30');
  });

  test('should calculate sum', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    await input.fill('[1, 2, 3, 4, 5].reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    await expect(output).toContainText('15');
  });

  test('should calculate sum of squares', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Sum of squares: [1, 2, 3, 4, 5] → [1, 4, 9, 16, 25] → 55
    await input.fill('[1, 2, 3, 4, 5].map(|x| x * x).reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    await expect(output).toContainText('55');
  });

  test('should calculate weighted average components', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Weighted sum: (value * weight) summed
    // Values: [80, 90, 85], Weights: [2, 3, 1]
    await input.fill('let data = [[80, 2], [90, 3], [85, 1]]');
    await input.press('Enter');

    await input.fill('data.map(|pair| pair[0] * pair[1]).reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    // Weighted sum = 515
    await expect(output).toContainText('515');
  });

  test('should access percentile via indexing', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Sorted data, median is middle element
    await input.fill('let sorted = [10, 20, 30, 40, 50]');
    await input.press('Enter');

    // Median: index 2 for 5 elements
    await input.fill('sorted[2]');
    await input.press('Enter');

    await expect(output).toContainText('30');
  });

  test('should calculate z-score components', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Z-score = (x - mean) / std_dev
    // Test deviation from mean
    await input.fill('let value = 30');
    await input.press('Enter');

    await input.fill('let mean = 20');
    await input.press('Enter');

    await input.fill('value - mean');
    await input.press('Enter');

    await expect(output).toContainText('10');
  });

  test('should calculate moving average', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Simple moving average (window size 3)
    await input.fill('let window = [10, 20, 30]');
    await input.press('Enter');

    await input.fill('window.reduce(|acc, x| acc + x, 0) / window.len()');
    await input.press('Enter');

    await expect(output).toContainText('20');
  });

  test('should calculate data normalization', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Min-max normalization
    await input.fill('let value = 50');
    await input.press('Enter');

    await input.fill('let min = 0');
    await input.press('Enter');

    await input.fill('let max = 100');
    await input.press('Enter');

    // Normalized (scaled by 10 for integer representation)
    await input.fill('((value - min) * 10) / (max - min)');
    await input.press('Enter');

    await expect(output).toContainText('5'); // 0.5 * 10
  });

  test('should detect outliers with threshold', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Outlier detection: mean + 2*threshold
    await input.fill('let mean = 28');
    await input.press('Enter');

    await input.fill('let threshold = 20');
    await input.press('Enter');

    await input.fill('mean + (2 * threshold)');
    await input.press('Enter');

    await expect(output).toContainText('68');
  });
});
