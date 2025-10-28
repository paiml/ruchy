import { test, expect } from '@playwright/test';

/**
 * Matrix Test 01: Simple Arithmetic Operations (Notebook Platform)
 *
 * This test verifies basic arithmetic works correctly in the Ruchy notebook interface.
 * Companion native test: tests/matrix/native/01_simple_arithmetic_native.rs
 *
 * Goal: Ensure identical behavior between notebook and native platforms
 *
 * FIXED: DEFECT-E2E-PHANTOM-UI - Rewritten to use actual notebook.html elements
 * Previous: Tests expected phantom UI (#status, #repl-input) that never existed
 * Current: Tests use real notebook UI (#notebook-cells, .CodeMirror, cell outputs)
 */

test.describe('Matrix Test 01: Simple Arithmetic (Notebook Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for actual notebook interface to load
    await page.waitForSelector('#notebook-cells', { timeout: 10000 });

    // Verify notebook UI components are ready
    await expect(page.locator('#notebook-cells')).toBeVisible();
    await expect(page.locator('.CodeMirror').first()).toBeVisible();
  });

  test('should compute 10 + 20 = 30', async ({ page }) => {
    // Get first CodeMirror editor
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Clear any existing content and type expression
    await page.keyboard.press('Control+A');
    await page.keyboard.type('10 + 20');

    // Execute cell with Shift+Enter
    await page.keyboard.press('Shift+Enter');

    // Wait for output to appear
    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('30');
  });

  test('should compute 100 - 42 = 58', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    await page.keyboard.press('Control+A');
    await page.keyboard.type('100 - 42');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('58');
  });

  test('should compute 6 * 7 = 42', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    await page.keyboard.press('Control+A');
    await page.keyboard.type('6 * 7');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('42');
  });

  test('should compute 100 / 4 = 25', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    await page.keyboard.press('Control+A');
    await page.keyboard.type('100 / 4');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('25');
  });

  test('should handle operator precedence: 2 + 3 * 4 = 14', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    await page.keyboard.press('Control+A');
    await page.keyboard.type('2 + 3 * 4');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    // Should respect precedence: 2 + (3 * 4) = 14, not (2 + 3) * 4 = 20
    await expect(output).toContainText('14');
  });

  test('should handle parentheses: (2 + 3) * 4 = 20', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    await page.keyboard.press('Control+A');
    await page.keyboard.type('(2 + 3) * 4');
    await page.keyboard.press('Shift+Enter');

    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('20');
  });

  test('should handle variables: let x = 10, x * 2 = 20', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Declare variable in first cell
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let x = 10');
    await page.keyboard.press('Shift+Enter');

    // Wait for first cell output
    await page.waitForTimeout(1000);

    // Add new cell for variable usage
    const addCellButton = page.locator('#btn-add-cell');
    await addCellButton.click();

    // Wait for new cell to be created
    await page.waitForTimeout(500);

    // Type in second cell
    const secondCell = page.locator('.CodeMirror').nth(1);
    await secondCell.click();
    await page.keyboard.type('x * 2');
    await page.keyboard.press('Shift+Enter');

    // Check second cell output
    const output = page.locator('.cell-output').nth(1);
    await expect(output).toBeVisible({ timeout: 5000 });
    await expect(output).toContainText('20');
  });

  test('should handle multi-step computation', async ({ page }) => {
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Define all variables and computation in single cell
    await page.keyboard.press('Control+A');
    await page.keyboard.type('let a = 5\nlet b = 10\nlet c = 15\na + b + c');
    await page.keyboard.press('Shift+Enter');

    // Wait for output
    const output = page.locator('.cell-output').first();
    await expect(output).toBeVisible({ timeout: 5000 });
    // Should be 5 + 10 + 15 = 30
    await expect(output).toContainText('30');
  });
});
