import { test, expect } from '@playwright/test';

/**
 * Matrix Test 03: Statistical Analysis (Notebook Platform)
 *
 * This test verifies statistical computation workflows in the Ruchy notebook interface.
 * Companion native test: tests/matrix_003_statistical_analysis_native.rs
 *
 * Goal: Ensure identical behavior between notebook and native platforms
 *
 * FIXED: DEFECT-E2E-PHANTOM-UI - Rewritten to use actual notebook.html elements
 * Previous: Tests expected phantom UI (#status, #repl-input) that never existed
 * Current: Tests use real notebook UI (#notebook-cells, .CodeMirror, cell outputs)
 */

test.describe('Matrix Test 03: Statistical Analysis (Notebook Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for actual notebook interface to load
    await page.waitForSelector('#notebook-cells', { timeout: 10000 });

    // Verify notebook UI components are ready
    await expect(page.locator('#notebook-cells')).toBeVisible();
    await expect(page.locator('.CodeMirror').first()).toBeVisible();
  });

  test('should calculate mean', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Create dataset and calculate mean
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let data = [10, 20, 30, 40, 50]\ndata.reduce(|acc, x| acc + x, 0) / data.len()');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    // Mean = 150 / 5 = 30
    await expect(output).toContainText('30');
  });

  test('should calculate sum', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    await page.keyboard.press('Control+A');
    await page.keyboard.type('[1, 2, 3, 4, 5].reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('15');
  });

  test('should calculate sum of squares', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Sum of squares: [1, 2, 3, 4, 5] → [1, 4, 9, 16, 25] → 55
    await page.keyboard.press('Control+A');
    await page.keyboard.type('[1, 2, 3, 4, 5].map(|x| x * x).reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('55');
  });

  test('should calculate weighted average components', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Weighted sum: (value * weight) summed
    // Values: [80, 90, 85], Weights: [2, 3, 1]
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let data = [[80, 2], [90, 3], [85, 1]]\ndata.map(|pair| pair[0] * pair[1]).reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    // Weighted sum = 515
    await expect(output).toContainText('515');
  });

  test('should access percentile via indexing', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Sorted data, median is middle element
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let sorted = [10, 20, 30, 40, 50]\nsorted[2]');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('30');
  });

  test('should calculate z-score components', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Z-score = (x - mean) / std_dev
    // Test deviation from mean
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let value = 30\nlet mean = 20\nvalue - mean');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('10');
  });

  test('should calculate moving average', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Simple moving average (window size 3)
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let window = [10, 20, 30]\nwindow.reduce(|acc, x| acc + x, 0) / window.len()');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('20');
  });

  test('should calculate data normalization', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Min-max normalization
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let value = 50\nlet min = 0\nlet max = 100\n((value - min) * 10) / (max - min)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('5'); // 0.5 * 10
  });

  test('should detect outliers with threshold', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Outlier detection: mean + 2*threshold
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let mean = 28\nlet threshold = 20\nmean + (2 * threshold)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('68');
  });
});
