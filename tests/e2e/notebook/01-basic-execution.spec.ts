import { test, expect } from '@playwright/test';

/**
 * NOTEBOOK-007: E2E Testing - Basic Cell Execution
 *
 * Tests basic notebook functionality across all 3 browsers:
 * - Chromium
 * - Firefox
 * - WebKit (Safari)
 */

test.describe('Basic Cell Execution', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to notebook interface
    await page.goto('/notebook.html');

    // Wait for WASM to load
    await page.waitForSelector('#notebook-container', { timeout: 10000 });
  });

  test('should load notebook interface', async ({ page }) => {
    // Verify essential UI elements are present
    await expect(page.locator('#notebook-container')).toBeVisible();
    await expect(page.locator('.cell-input')).toBeVisible();
    await expect(page.locator('.execute-button')).toBeVisible();
  });

  test('should execute simple arithmetic', async ({ page }) => {
    // Type code into cell
    await page.fill('.cell-input', '2 + 2');

    // Execute cell
    await page.click('.execute-button');

    // Wait for output
    await page.waitForSelector('.cell-output', { timeout: 5000 });

    // Verify output
    const output = await page.textContent('.cell-output');
    expect(output).toContain('4');
  });

  test('should execute variable assignment', async ({ page }) => {
    // Execute first cell: variable assignment
    await page.fill('.cell-input', 'let x = 10');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    // Add new cell
    await page.click('.add-cell-button');

    // Execute second cell: use variable
    const cells = await page.locator('.cell-input').all();
    await cells[1].fill('x * 2');
    await page.locator('.execute-button').nth(1).click();

    // Verify output
    await page.waitForSelector('.cell-output:nth-child(2)');
    const output = await page.locator('.cell-output').nth(1).textContent();
    expect(output).toContain('20');
  });

  test('should display syntax errors', async ({ page }) => {
    // Enter invalid syntax
    await page.fill('.cell-input', 'let x = ');
    await page.click('.execute-button');

    // Wait for error message
    await page.waitForSelector('.cell-error', { timeout: 5000 });

    // Verify error is displayed
    const error = await page.textContent('.cell-error');
    expect(error).toBeTruthy();
    expect(error?.length).toBeGreaterThan(0);
  });

  test('should persist state across cells', async ({ page }) => {
    // Cell 1: Define function
    await page.fill('.cell-input', 'fn add(a, b) { a + b }');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    // Cell 2: Call function
    await page.click('.add-cell-button');
    const cells = await page.locator('.cell-input').all();
    await cells[1].fill('add(5, 3)');
    await page.locator('.execute-button').nth(1).click();

    // Verify output
    await page.waitForTimeout(1000);
    const output = await page.locator('.cell-output').nth(1).textContent();
    expect(output).toContain('8');
  });
});

test.describe('Cell Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('should add new cells', async ({ page }) => {
    const initialCells = await page.locator('.cell').count();

    // Add new cell
    await page.click('.add-cell-button');

    // Verify cell was added
    const newCells = await page.locator('.cell').count();
    expect(newCells).toBe(initialCells + 1);
  });

  test('should delete cells', async ({ page }) => {
    // Add a cell first
    await page.click('.add-cell-button');
    const cellCount = await page.locator('.cell').count();

    // Delete the last cell
    await page.locator('.delete-cell-button').last().click();

    // Verify cell was deleted
    const newCount = await page.locator('.cell').count();
    expect(newCount).toBe(cellCount - 1);
  });

  test('should reorder cells', async ({ page }) => {
    // Add two cells with different content
    await page.fill('.cell-input', 'let first = 1');
    await page.click('.add-cell-button');

    const cells = await page.locator('.cell-input').all();
    await cells[1].fill('let second = 2');

    // Move second cell up
    await page.locator('.move-up-button').nth(1).click();

    // Verify order changed
    const newCells = await page.locator('.cell-input').all();
    const firstContent = await newCells[0].inputValue();
    expect(firstContent).toContain('second');
  });
});
