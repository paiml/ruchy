import { test, expect } from '@playwright/test';

/**
 * Matrix Test 02: CSV Processing Workflow (WASM Platform) - DEFERRED
 *
 * This test verifies data processing workflows in the WASM REPL.
 * Companion native test: tests/matrix_002_csv_workflow_native.rs
 *
 * STATUS: DEFERRED pending WASM eval() implementation
 * See: tests/e2e/matrix/README.md for rationale
 *
 * Goal: Ensure identical behavior between WASM and native platforms
 */

test.describe('Matrix Test 02: CSV Processing Workflow (WASM Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to load
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });
  });

  test('should create and access array data', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Create array simulating CSV rows
    await input.fill('let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]');
    await input.press('Enter');

    // Verify array length
    await input.fill('data.len()');
    await input.press('Enter');

    await expect(output).toContainText('3');
  });

  test('should filter array data', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Create data
    await input.fill('let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]');
    await input.press('Enter');

    // Filter: age > 30 (index 1)
    await input.fill('let filtered = data.filter(|row| row[1] > 30)');
    await input.press('Enter');

    // Verify filtered length
    await input.fill('filtered.len()');
    await input.press('Enter');

    await expect(output).toContainText('2'); // Age 35 and 45
  });

  test('should map array data', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Create data
    await input.fill('let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]');
    await input.press('Enter');

    // Map: extract salaries (index 2)
    await input.fill('let salaries = data.map(|row| row[2])');
    await input.press('Enter');

    // Verify first salary
    await input.fill('salaries[0]');
    await input.press('Enter');

    await expect(output).toContainText('50000');
  });

  test('should aggregate with reduce', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Create simple numeric array
    await input.fill('let numbers = [10, 20, 30, 40, 50]');
    await input.press('Enter');

    // Sum using reduce
    await input.fill('let sum = numbers.reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    // Verify sum
    await input.fill('sum');
    await input.press('Enter');

    await expect(output).toContainText('150');
  });

  test('should perform complete filter-map-reduce workflow', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Complete workflow in one chained expression
    await input.fill('[[1, 25, 50000], [2, 35, 75000], [3, 45, 100000], [4, 32, 80000]].filter(|row| row[1] > 30).map(|row| row[2]).reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    // Should be: 75000 + 100000 + 80000 = 255000
    await expect(output).toContainText('255000');
  });

  test('should handle nested data structures', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Simulate CSV with headers via struct-like objects
    await input.fill('let row1 = {"id": 1, "name": "Alice", "age": 30}');
    await input.press('Enter');

    // Access field
    await input.fill('row1.age');
    await input.press('Enter');

    await expect(output).toContainText('30');
  });

  test('should chain multiple operations', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Create array
    await input.fill('let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]');
    await input.press('Enter');

    // Chain: filter even numbers, map to squares, sum
    await input.fill('data.filter(|x| x % 2 == 0).map(|x| x * x).reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    // Even numbers: 2,4,6,8,10 → squares: 4,16,36,64,100 → sum: 220
    await expect(output).toContainText('220');
  });

  test('should handle real-world data pipeline', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Simulate sales data: [product_id, quantity, price]
    await input.fill('let sales = [[101, 5, 10], [102, 3, 20], [103, 8, 15], [104, 2, 25]]');
    await input.press('Enter');

    // Calculate total revenue: quantity * price for each row
    await input.fill('let revenues = sales.map(|row| row[1] * row[2])');
    await input.press('Enter');

    // Sum total revenue
    await input.fill('revenues.reduce(|acc, x| acc + x, 0)');
    await input.press('Enter');

    // (5*10) + (3*20) + (8*15) + (2*25) = 50 + 60 + 120 + 50 = 280
    await expect(output).toContainText('280');
  });
});
