import { test, expect } from '@playwright/test';

/**
 * Matrix Test 02: CSV Processing Workflow (Notebook Platform)
 *
 * This test verifies data processing workflows in the Ruchy notebook interface.
 * Companion native test: tests/matrix_002_csv_workflow_native.rs
 *
 * Goal: Ensure identical behavior between notebook and native platforms
 *
 * FIXED: DEFECT-E2E-PHANTOM-UI - Rewritten to use actual notebook.html elements
 * Previous: Tests expected phantom UI (#status, #repl-input) that never existed
 * Current: Tests use real notebook UI (#notebook-cells, .CodeMirror, cell outputs)
 */

test.describe('Matrix Test 02: CSV Processing Workflow (Notebook Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for actual notebook interface to load
    await page.waitForSelector('#notebook-cells', { timeout: 10000 });

    // Verify notebook UI components are ready
    await expect(page.locator('#notebook-cells')).toBeVisible();
    await expect(page.locator('.CodeMirror').first()).toBeVisible();
  });

  test('should create and access array data', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Create array simulating CSV rows
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]\ndata.len()');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('3');
  });

  test('should filter array data', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Create data and filter in single cell
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]\nlet filtered = data.filter(|row| row[1] > 30)\nfiltered.len()');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('2'); // Age 35 and 45
  });

  test('should map array data', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Create data, map, and access in single cell
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]\nlet salaries = data.map(|row| row[2])\nsalaries[0]');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('50000');
  });

  test('should aggregate with reduce', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Create array and reduce in single cell
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let numbers = [10, 20, 30, 40, 50]\nnumbers.reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('150');
  });

  test('should perform complete filter-map-reduce workflow', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Complete workflow in one chained expression
    await page.keyboard.press('Control+A');
    await page.keyboard.type('[[1, 25, 50000], [2, 35, 75000], [3, 45, 100000], [4, 32, 80000]].filter(|row| row[1] > 30).map(|row| row[2]).reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    // Should be: 75000 + 100000 + 80000 = 255000
    await expect(output).toContainText('255000');
  });

  test('should handle nested data structures', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Simulate CSV with headers via struct-like objects
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let row1 = {"id": 1, "name": "Alice", "age": 30}\nrow1.age');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('30');
  });

  test('should chain multiple operations', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Chain: filter even numbers, map to squares, sum
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]\ndata.filter(|x| x % 2 == 0).map(|x| x * x).reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    // Even numbers: 2,4,6,8,10 → squares: 4,16,36,64,100 → sum: 220
    await expect(output).toContainText('220');
  });

  test('should handle real-world data pipeline', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Multi-step pipeline: create data, calculate revenues, sum
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let sales = [[101, 5, 10], [102, 3, 20], [103, 8, 15], [104, 2, 25]]\nlet revenues = sales.map(|row| row[1] * row[2])\nrevenues.reduce(|acc, x| acc + x, 0)');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    // (5*10) + (3*20) + (8*15) + (2*25) = 50 + 60 + 120 + 50 = 280
    await expect(output).toContainText('280');
  });
});
